pub use {
    csv,
    std::io::{self, Error, ErrorKind, Result},
};

use {
    crate::{Event, Schedule},
    std::str::FromStr,
};

#[derive(Debug)]
pub struct EventModel {
    pub name: Box<str>,
    pub leader: Option<Box<str>>,

    // #[serde(default = "EventModel::default_len")]
    pub len: usize,
}

impl ToString for EventModel {
    fn to_string(&self) -> String {
        let base = match &self.leader {
            Some(leader) => format!("{}:{}", self.name, leader),
            None => self.name.to_string(),
        };
        if self.len != 1 {
            format!("{}[{}]", base, self.len)
        } else {
            base
        }
    }
}

impl FromStr for EventModel {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut s = String::from(s);
        let mut len = 1;
        if s.chars().last().ok_or(Error::new(
            ErrorKind::InvalidInput,
            "missing field: \"event\"",
        ))? == ']'
        {
            let (other, len_s) =
                s.get(0..s.len() - 1)
                    .unwrap()
                    .split_once("[")
                    .ok_or(Error::new(
                        ErrorKind::InvalidInput,
                        "missing field: \"len\"",
                    ))?;
            len = len_s.trim().parse::<usize>().map_err(|_| {
                Error::new(
                    ErrorKind::InvalidData,
                    "field \"len\" expected type: <integer>",
                )
            })?;

            s = other.trim().to_string();
        }

        let event = if let Some((name, leader)) = s.split_once(':') {
            let name = name.trim();
            let leader = leader.trim();

            if name.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "missing field: \"name\"",
                ));
            }
            if leader.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "missing field: \"leader\"",
                ));
            }

            EventModel {
                name: Box::from(name),
                leader: Some(Box::from(leader)),
                len,
            }
        } else {
            if s.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "missing field: \"name\"",
                ));
            }
            EventModel {
                name: Box::from(s),
                leader: None,
                len,
            }
        };

        Ok(event)
    }
}

impl Into<Event> for EventModel {
    fn into(self) -> Event {
        Event::new(self.name, self.leader, self.len)
    }
}

impl From<Event> for EventModel {
    fn from(e: Event) -> Self {
        EventModel {
            name: Box::from(e.name.as_ref()),
            leader: e.leader_name.map(|l| Box::from(l.as_ref())),
            len: e.len,
        }
    }
}

#[derive(Debug)]
pub struct ScheduleModel(Vec<Vec<EventModel>>);

impl Into<Vec<Vec<Event>>> for ScheduleModel {
    fn into(self) -> Vec<Vec<Event>> {
        self.0
            .into_iter()
            .map(|i| i.into_iter().map(|e| e.into()).collect())
            .collect()
    }
}

impl From<Schedule> for ScheduleModel {
    fn from(s: Schedule) -> Self {
        ScheduleModel(
            s.scheme
                .into_iter()
                .map(|i| i.into_iter().map(|e| e.into()).collect())
                .collect(),
        )
    }
}

impl ScheduleModel {
    pub fn serialize_csv<W>(self, writer: &mut csv::Writer<W>) -> csv::Result<()>
    where
        W: std::io::Write,
    {
        for line in self.0 {
            for event in line {
                writer.write_field(event.to_string())?;
            }
            writer.write_record(None::<&[u8]>)?;
        }

        Ok(())
    }

    pub fn deserialize_csv<R>(reader: &mut csv::Reader<R>) -> io::Result<Self>
    where
        R: io::Read,
    {
        let scheme: Vec<Vec<EventModel>> = reader
            .deserialize()
            .map(|line| {
                let line: Vec<String> = line.unwrap();
                line.into_iter()
                    .map(|e| EventModel::from_str(&e))
                    .collect::<Result<Vec<EventModel>>>()
            })
            .collect::<Result<Vec<Vec<EventModel>>>>()?;
        Ok(Self(scheme))
    }
}
