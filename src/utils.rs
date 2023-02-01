// pub type PoolPostgres = sqlx::pool::PoolConnection<sqlx::Postgres>;

pub mod default_date_format {
    use chrono::{Local, NaiveDateTime};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // if undefined, return None
        let s: Option<String> = match Deserialize::deserialize(deserializer) {
            Ok(s) => s,
            Err(err) => return Err(serde::de::Error::custom(err)),
        };

        if s.is_none() {
            return Ok(None);
        }

        let dt = match NaiveDateTime::parse_from_str(&s.unwrap(), FORMAT) {
            Ok(dt) => dt,
            Err(err) => return Err(serde::de::Error::custom(err)),
        };

        let dt = dt.and_local_timezone(Local).unwrap();

        Ok(Some(dt.naive_utc()))
    }
}
