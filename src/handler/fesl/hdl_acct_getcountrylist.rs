use indexmap::IndexMap;

use crate::handler::submit_packet;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;


pub async fn acct_getcountrylist(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    let mut response_hm: IndexMap<_, _, _> = IndexMap::new();

    response_hm.insert("TXN".to_string(), "GetCountryList".to_string());

    // Found ISO2 country codes that worked for LOTR:CQ
    const FESL_COUNTRY_LIST: [[(&str, &str); 5]; 17] = [
        [
            ("ISOCode", "AU"),
            ("description", "Australia"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "BE"),
            ("description", "Belgium"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "CA"),
            ("description", "Canada"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "DK"),
            ("description", "Denmark"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "FI"),
            ("description", "Finland"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "FR"),
            ("description", "France"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "IE"),
            ("description", "Ireland"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "IT"),
            ("description", "Italy"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "NL"),
            ("description", "Netherlands"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "NO"),
            ("description", "Norway"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "PL"),
            ("description", "Poland"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "PT"),
            ("description", "Portugal"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "RU"),
            ("description", "Russian Federation"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "ES"),
            ("description", "Spain"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "SE"),
            ("description", "Sweden"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "GB"),
            (
                "description",
                "United Kingdom of Great Britain and Northern Ireland",
            ),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
        [
            ("ISOCode", "US"),
            ("description", "United States of America"),
            ("allowEmailsDefaultValue", "1"),
            ("parentalControlAgeLimit", "1"),
            ("registrationAgeLimit", "1"),
        ],
    ];

    for (idx, country) in FESL_COUNTRY_LIST.iter().enumerate() {
        for (key, value) in country.iter() {
            // The list index starts with 1 in the EA format
            response_hm.insert(
                format!("countryList.{}.{}", idx + 0, key),
                value.to_string(),
            );
        }
    }
    response_hm.insert(
        "countryList.[]".to_string(),
        FESL_COUNTRY_LIST.len().to_string(),
    );

    let response = DataPacket::new(
        DataMode::FESL_ACCT,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
