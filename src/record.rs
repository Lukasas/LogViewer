use chrono::{DateTime, Local};

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub id: u64,
    pub date: DateTime<Local>,
    pub computer_name: String,
    pub process_id: u32,
    pub process_user: String,
    pub module_name: String,
    pub message_id: u32,
    pub log_level: u8,
    pub facility: u32,
    pub log_message: String,
}

impl Record {
    pub fn read_records() -> std::io::Result<Vec<Record>> {
        Ok(vec![Record {
            id: 0,
            date: Local::now(),
            computer_name: String::from("My PC"),
            process_id: 1234,
            process_user: String::from("My User"),
            module_name: String::from("Hello"),
            message_id: 1,
            log_level: 3,
            facility: 5,
            log_message: String::from("This is a log message 1"),
        }])
    }
}

#[cfg(test)]

mod tests {
    use crate::*;

    #[test]
    fn test_load_small_file() {
        let events = Record::read_records();
        assert_eq!(events.is_ok(), true);
    }
}
