use crate::client_connection::{ClientConnectionDescriptor, ProtoType, SendDataType, ServiceType};
use crate::orm::model::{account, game, participant, persona, session};
use crate::packet::DataPacket;
use crate::sharedstate::SharedState;
use core::panic;
use std::sync::Arc;
//use crate::plasma_errors::EAError;
use crate::mordorwide_errors::MWErr;

use crate::utils::auth::user::{get_credentials_from_packet, validate_credentials};

use sea_orm::entity::*;
use sea_orm::query::*;

pub struct PlasmaRequestBundle {
    pub packet: DataPacket,
    pub con: ClientConnectionDescriptor,
    pub sstate: Arc<SharedState>,
    session_model: Option<session::Model>,
    persona_model: Option<persona::Model>,
    user_model: Option<account::Model>,
}

impl PlasmaRequestBundle {
    pub fn new(
        packet: DataPacket,
        con: ClientConnectionDescriptor,
        sstate: Arc<SharedState>,
    ) -> Self {
        Self {
            packet,
            con,
            sstate,
            session_model: None,
            persona_model: None,
            user_model: None,
        }
    }

    pub fn flush(&mut self) {
        self.session_model = None;
        self.persona_model = None;
        self.user_model = None;
    }

    async fn clear_active_session(&mut self, session: session::Model) {
        let persona_id = session.persona_id;

        if persona_id != -1 {
            // Clear participant entries
            let Ok(participants) = participant::Entity::find()
                .filter(participant::Column::PersonaId.eq(persona_id))
                .all(&*self.sstate.database)
                .await
            else {
                panic!("Failed to clear participants");
            };

            // Find all associated games
            let Ok(games) = game::Entity::find()
                .filter(game::Column::PersonaId.eq(persona_id))
                .all(&*self.sstate.database)
                .await
            else {
                panic!("Failed to find games");
            };
            let game_ids = games.iter().map(|g| g.id).collect::<Vec<i64>>();

            // Clear participants to the persona-owned games
            for game_id in game_ids {
                let Ok(_) = participant::Entity::delete_many()
                    .filter(participant::Column::GameId.eq(game_id))
                    .exec(&*self.sstate.database)
                    .await
                else {
                    panic!("Failed to clear participants");
                };
            }

            // Clear games
            let Ok(_) = game::Entity::delete_many()
                .filter(game::Column::PersonaId.eq(persona_id))
                .exec(&*self.sstate.database)
                .await
            else {
                panic!("Failed to clear games");
            };
        }

        // Terminate TCP connections (TCP+FESL)
        if session.fesl_tcp_handle != "" {
            let con_descr = ClientConnectionDescriptor::from_string(&session.fesl_tcp_handle);
            {
                let conns = &*self.sstate.connections;
                if conns.contains_key(&con_descr) {
                    if let Some(tcp_con) = conns.get_mut(&con_descr) {
                        tcp_con.send(SendDataType::Close).await;
                    };
                    conns.remove(&con_descr);
                }
            }
        }
        // Terminate TCP connections (TCP+THEATER)
        if session.theater_tcp_handle != "" {
            let con_descr = ClientConnectionDescriptor::from_string(&session.theater_tcp_handle);
            {
                let conns = &*self.sstate.connections;
                if conns.contains_key(&con_descr) {
                    if let Some(tcp_con) = conns.get_mut(&con_descr) {
                        tcp_con.send(SendDataType::Close).await;
                    };
                    conns.remove(&con_descr);
                }
            }
        }

        // Clear session
        let Ok(_) = session::Entity::delete_by_id(session.id)
            .exec(&*self.sstate.database)
            .await
        else {
            panic!("Failed to clear session");
        };
        self.flush();
    }

    async fn clear_active_sessions_by_user(&mut self, user_id: i64, except: Option<i64>) {
        let Ok(sessions) = session::Entity::find()
            .filter(session::Column::UserId.eq(user_id))
            .all(&*self.sstate.database)
            .await
        else {
            panic!("Failed to find sessions");
        };

        for session in sessions {
            if except.is_some() && except.unwrap() == session.id {
                continue;
            }
            self.clear_active_session(session).await;
        }
    }

    pub async fn get_active_session_model(&mut self) -> Option<session::Model> {
        if self.session_model.is_some() {
            return self.session_model.clone();
        }

        if self.con.proto_type == ProtoType::Tcp && self.con.service_type == ServiceType::Fesl {
            let Ok(Some(session)) = session::Entity::find()
                .filter(session::Column::FeslTcpHandle.eq(self.con.to_string()))
                .one(&*self.sstate.database)
                .await
            else {
                return None;
            };
            self.session_model = Some(session.clone());
            return Some(session);
        } else if self.con.proto_type == ProtoType::Tcp
            && self.con.service_type == ServiceType::Theater
        {
            let Ok(Some(session)) = session::Entity::find()
                .filter(session::Column::TheaterTcpHandle.eq(self.con.to_string()))
                .one(&*self.sstate.database)
                .await
            else {
                return None;
            };
            self.session_model = Some(session.clone());
            return Some(session);
        } else if self.con.proto_type == ProtoType::Udp
            && self.con.service_type == ServiceType::Theater
        {
            let Ok(Some(session)) = session::Entity::find()
                .filter(session::Column::TheaterUdpHandle.eq(self.con.to_string()))
                .one(&*self.sstate.database)
                .await
            else {
                return None;
            };
            self.session_model = Some(session.clone());
            return Some(session);
        } else {
            panic!("Invalid connection type");
        }
    }

    pub async fn get_active_persona_model(&mut self) -> Option<persona::Model> {
        if self.persona_model.is_some() {
            return self.persona_model.clone();
        }

        let session = self.get_active_session_model().await;
        if let Some(session) = session {
            if session.persona_id == -1 {
                return None;
            } else {
                let Ok(Some(persona)) = persona::Entity::find()
                    .filter(persona::Column::Id.eq(session.persona_id))
                    .one(&*self.sstate.database)
                    .await
                else {
                    return None;
                };
                self.persona_model = Some(persona.clone());
                return Some(persona);
            }
        }
        return None;
    }

    pub async fn get_active_user_model(&mut self) -> Option<account::Model> {
        if self.user_model.is_some() {
            return self.user_model.clone();
        }

        let session = self.get_active_session_model().await;
        if let Some(session) = session {
            let Ok(Some(account)) = account::Entity::find()
                .filter(account::Column::Id.eq(session.user_id))
                .one(&*self.sstate.database)
                .await
            else {
                return None;
            };
            self.user_model = Some(account.clone());
            return Some(account);
        }
        return None;
    }

    pub async fn set_active_user_session(
        &mut self,
        lobby_key: &String,
        user_id: i64,
        except: Option<i64>,
    ) -> bool {
        if self.con.proto_type != ProtoType::Tcp || self.con.service_type != ServiceType::Fesl {
            return false;
        }
        // Clear older sessions first
        self.clear_active_sessions_by_user(user_id, except).await;

        // Insert new session
        let mut session = session::ActiveModel {
            lobby_key: Set(lobby_key.clone()),
            user_id: Set(user_id),
            persona_id: Set(-1),
            fesl_tcp_handle: Set(self.con.to_string()),
            theater_tcp_handle: Set("".to_string()),
            theater_udp_handle: Set("".to_string()),
            nat_type: Set(0),
            ..Default::default()
        };

        // Check if the session should be re-used?
        if let Some(old_session_id) = except {
            if let Ok(Some(old_session)) = session::Entity::find_by_id(old_session_id)
                .one(&*self.sstate.database)
                .await
            {
                session.id = Set(old_session_id);
                // The FESL connection should be in-fact identical to the previous one.
                // Play it safe and use the new handles though...
                // fesl_tcp_handle = Set(old_session.fesl_tcp_handle);
                session.theater_tcp_handle = Set(old_session.theater_tcp_handle);
                session.theater_udp_handle = Set(old_session.theater_udp_handle);
            };
        }
        // Now, write the session to the database
        let Ok(_) = session.save(&*self.sstate.database).await else {
            return false;
        };

        // Update the login date of the account model
        let Ok(Some(db_user)) = account::Entity::find_by_id(user_id)
            .one(&*self.sstate.database)
            .await
        else {
            return false;
        };
        let mut db_user = db_user.into_active_model();
        db_user.last_login = Set(chrono::Utc::now());
        let Ok(_) = db_user.save(&*self.sstate.database).await else {
            return false;
        };

        self.flush();

        return true;
    }

    pub async fn set_active_persona_session(&mut self, persona_id: i64) -> bool {
        if self.con.proto_type != ProtoType::Tcp || self.con.service_type != ServiceType::Fesl {
            return false;
        }
        let session = self.get_active_session_model().await;

        if let Some(session) = session {
            let mut session = session.into_active_model();
            session.persona_id = Set(persona_id);

            if let Ok(_) = session.save(&*self.sstate.database).await {
                self.flush();
                return true;
            };
        }
        return false;
    }

    pub async fn is_authenticated_user(&mut self) -> bool {
        self.get_active_session_model().await.is_some()
    }

    pub async fn is_authenticated_user_and_persona(&mut self) -> bool {
        if let Some(session) = self.get_active_persona_model().await {
            return session.id != -1;
        }
        return false;
    }

    pub async fn auth_by_packet(&mut self) -> Result<(), MWErr> {
        // You cannot assume that the current session is NOT authenticated.
        // If you register a new user, the game immediately logs in with the new user.
        // This means that the session is already authenticated.
        // However, if the user is not entitled and asks for it, the game will attempt to send the entitlement key.
        // Afterwards, it will again attempt to login again.
        // -> This means that the session can already be authenticated.
        let active_session = self.get_active_session_model().await;

        // Check if a login packet is present
        let is_login_pkt = ["NuLogin", "NuPS3Login", "NuXBL360Login"].contains(
            &self
                .packet
                .data
                .get("TNX")
                .unwrap_or(&"".to_string())
                .as_str(),
        );

        let credentials = get_credentials_from_packet(&self.packet, &self.sstate).await;
        if credentials.is_err() {
            return Err(credentials.unwrap_err());
        }
        let credentials = credentials.unwrap();

        let validation = validate_credentials(&credentials, &self.sstate).await;
        if validation.is_err() {
            return Err(validation.unwrap_err());
        }
        let user_id = validation.unwrap();

        // ToDo: Make the lobby key handling more generic
        let Ok(Some(user_db)) = account::Entity::find_by_id(user_id)
            .one(&*self.sstate.database)
            .await
        else {
            panic!("Failed to find user that already has been validated earlier.");
        };
        let lobby_key = user_db.lobby_key;

        // Register active session
        let except = if active_session.is_some() {
            Some(active_session.unwrap().id)
        } else {
            None
        };
        let success = self
            .set_active_user_session(&lobby_key, user_id, except)
            .await;

        if !success {
            panic!("Failed to register active session");
        }

        Ok(())
    }
}
