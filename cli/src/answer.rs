use std::{borrow::Cow, fmt::Display};

pub enum Answer {
    Error(String),
    Successfully(Option<String>),
    Data(String),
    Invalid,
}

impl From<&str> for Answer {
    // dry
    fn from(value: &str) -> Self {
        let mut as_array: Vec<&str> = value.split_whitespace().collect();
        let command = as_array.remove(0);
        match command {
            "error" => Answer::Error(as_array.join(" ")),
            "successfully" => Answer::Successfully(if as_array.is_empty() {
                None
            } else {
                Some(as_array.join(" "))
            }),
            "data" => Answer::Data(as_array.join(" ")),
            _ => Answer::Invalid,
        }
    }
}

impl From<Cow<'_, str>> for Answer {
    // dry
    fn from(value: Cow<'_, str>) -> Self {
        let mut as_array: Vec<&str> = value.split_whitespace().collect();
        let command = as_array.remove(0);
        match command {
            "error" => Answer::Error(as_array.join(" ")),
            "success" => Answer::Successfully(if as_array.is_empty() {
                None
            } else {
                Some(as_array.join(" "))
            }),
            "data" => Answer::Data(as_array.join(" ")),
            _ => Answer::Invalid,
        }
    }
}

impl Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(text) => {
                write!(f, "Error: {}", text)
            }
            Self::Successfully(text) => {
                write!(
                    f,
                    "Successfully{}",
                    if let Some(text) = text {
                        format!(": {text}")
                    } else {
                        "".to_string()
                    }
                )
            }
            Self::Data(data) => {
                write!(f, "{}", data)
            }
            Self::Invalid => {
                write!(f, "Invalid")
            }
        }
    }
}
