use super::{ConfigError, ConfigField, FieldError, FieldErrorKind};

fn field_error(field: ConfigField, kind: FieldErrorKind) -> Option<FieldError> {
    Some(FieldError {
        field: Some(field),
        kind,
    })
}

#[test]
fn validation_errors_map_to_fields_and_kinds() {
    let cases = [
        (
            ConfigError::InvalidHost(String::new()),
            field_error(ConfigField::Host, FieldErrorKind::Format),
        ),
        (
            ConfigError::InvalidCupsQueue(String::new()),
            field_error(ConfigField::CupsQueue, FieldErrorKind::Format),
        ),
        (
            ConfigError::InvalidScanDir(String::new()),
            field_error(ConfigField::ScanDir, FieldErrorKind::Format),
        ),
        (
            ConfigError::InvalidNotify("notify.tonerPercent is too high".to_string()),
            field_error(ConfigField::NotifyTonerPercent, FieldErrorKind::Range),
        ),
        (
            ConfigError::InvalidNotify("notify.drumPercent is too high".to_string()),
            field_error(ConfigField::NotifyDrumPercent, FieldErrorKind::Range),
        ),
        (
            ConfigError::InvalidToner(String::new()),
            field_error(ConfigField::TonerYieldPages, FieldErrorKind::Range),
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.field_error(), expected);
    }
}

#[test]
fn startup_errors_do_not_map_to_fields() {
    let errors = [
        ConfigError::Io(std::io::Error::other("disk failed")),
        ConfigError::Serde(serde_json::from_str::<serde_json::Value>("{").unwrap_err()),
        ConfigError::NoConfigDir,
        ConfigError::NoHomeDir,
    ];

    for error in errors {
        assert_eq!(error.field_error(), None);
    }
}

#[test]
fn field_errors_serialize_with_camel_case_tags() {
    let error = FieldError {
        field: Some(ConfigField::Host),
        kind: FieldErrorKind::Format,
    };
    assert_eq!(
        serde_json::to_string(&error).unwrap(),
        r#"{"field":"host","kind":"format"}"#
    );

    let error = FieldError {
        field: None,
        kind: FieldErrorKind::Io,
    };
    assert_eq!(
        serde_json::to_string(&error).unwrap(),
        r#"{"field":null,"kind":"io"}"#
    );

    let fields = [
        (ConfigField::Host, "host"),
        (ConfigField::PollSeconds, "pollSeconds"),
        (ConfigField::ScanDir, "scanDir"),
        (ConfigField::NotifyTonerPercent, "notifyTonerPercent"),
        (ConfigField::NotifyDrumPercent, "notifyDrumPercent"),
        (ConfigField::TonerYieldPages, "tonerYieldPages"),
        (ConfigField::CupsQueue, "cupsQueue"),
    ];
    for (field, expected) in fields {
        assert_eq!(
            serde_json::to_string(&field).unwrap(),
            format!("\"{expected}\"")
        );
    }

    let kinds = [
        (FieldErrorKind::Required, "required"),
        (FieldErrorKind::Format, "format"),
        (FieldErrorKind::Range, "range"),
        (FieldErrorKind::Io, "io"),
    ];
    for (kind, expected) in kinds {
        assert_eq!(
            serde_json::to_string(&kind).unwrap(),
            format!("\"{expected}\"")
        );
    }
}
