#[derive(Debug, Clone)]
pub enum EAError {
    //EA_ERROR = -12,
    EA_OK = 0,                       // Erfolgreich
    EA_TooYoung = -165, // Du erfüllst leider nicht die für die Registierung erforderlichen Vorgaben.
    EA_LoginErrorHeading = 21, // Bei der Anmeldung deines Kontos sind folgende Fehler aufgetreten: Fehlende oder ungültige Informationen. Bitte überprüfe deine Eingaben und versuche es erneut.
    EA_AuthFail = 100, // Die Authentifizierung dieses Kontos ist aus unbekannten Gründen fehlgeschlagen.
    EA_NotFound = 101, // Der angegebene Konto-Name konnte nicht gefunden werden.
    EA_Disabled = 102, // Der EA Nation-Zugang dieses Kontos wurde deaktiviert.
    EA_Banned = 103,   // Der EA Nation-Zugang dieses Kontos wurde gebannt.
    EA_NoData = 104,   // Die für diese Transaktion benötigten Daten konnten nicht gefunden werden.
    EA_Pending = 105,  // Der EA Nation-Zugang dieses Kontos ist in Bearbeitung.
    EA_Tentative = 107, // Dies ist ein vorläufiges Konto. Melde dich an, um weitere Informationen zu erhalten.
    EA_Parental_verification = 108, // Vorgang erfordert die Zustimmung eines Erziehungsberechtigten.
    EA_NotEntitled = 120, // Der Spieler ist nicht berechtigt, dieses Spiel online zu spielen.
    EA_TooManyAttempts = 121, // Zu viele Anmeldungsversuche mit einem ungültigen Konto. Falls du deine Anmeldedaten vergessen hast, besuche bitte http://profile.ea.com, um Unterstützung zu erlangen, wie du deinen Benutzernamen und dein Passwort wiederherstellen kannst.
    EA_InvalidPassword = 122, // Der Benutzer hat ein ungültiges Passwort angegeben.
    EA_NotRegistered = 123,   // Der Benutzer hat dieses Spiel nicht registriert.
    EA_TooManyPassRecov = 140, // Der Benutzer hat das Passwort zu oft angefordert.
    EA_TooManyNameRecov = 141, // Der Benutzer hat den Kontonamen zu oft angefordert.
    EA_EmailNotFound = 142,   // Die angegebene E-Mail-Adresse ist nicht in der Datenbank.
    EA_PasswordNotFound = 143, // Für das angegebene Konto existiert kein Passwort.
    EA_NameInUse = 160, // Der angegebene Kontoname wird bereits verwendet (ist bereits in der Datenbank).
    EA_EmailBlocked = 161, // Die angegebene E-Mail-Adresse wurde für die Erstellung neuer Konten gesperrt.
    EA_PasswordNotChanged = 162, // Das Passwort wurde aus unbekannten Gründen nicht geändert.
    EA_TooManyPersonas = 163, // Das Hauptkonto verfügt bereits über die maximale Zahl von Unterkonten.
    EA_RegCodeAlreadyInuse = 180, // Der Registrierungscode wird bereits verwendet.
    EA_InvalidRegCode = 181,  // Der eingegebene Registrierungscode ist ungültig.
    EA_AccountAlreadyEntitled = 182, // Das Konto ist bereits freigeschaltet.
    EA_AccountDeactivated = 250, // Für dieses Konto wurde der Zugriff auf EA Nation deaktiviert.
    EA_NewToS = 260,
}
