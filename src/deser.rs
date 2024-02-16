#![cfg(feature = "serde")]
#![cfg_attr(docsrs, doc(cfg(feature = "serde")))]
use super::GHRepo;
use serde::{
    de::{Deserializer, Unexpected, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};
use std::fmt;

impl Serialize for GHRepo {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.fullname)
    }
}

impl<'de> Deserialize<'de> for GHRepo {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct GHRepoVisitor;

        impl Visitor<'_> for GHRepoVisitor {
            type Value = GHRepo;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(
                    "a GitHub repository of the form OWNER/NAME or a GitHub repository URL",
                )
            }

            fn visit_str<E>(self, input: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                input
                    .parse::<GHRepo>()
                    .map_err(|_| E::invalid_value(Unexpected::Str(input), &self))
            }
        }

        deserializer.deserialize_str(GHRepoVisitor)
    }
}
