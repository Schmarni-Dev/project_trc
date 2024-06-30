use std::fmt::Display;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

use crate::remote_control_packets::RcC2SPacket;
use crate::remote_control_packets::RcS2CPacket;
use crate::remote_control_packets::RcS2TPacket;
use crate::remote_control_packets::RcT2SPacket;

// #[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
// pub struct ExtensionPacket<E: Copy + Clone> {
//     extension: E,
// }
//
// #[derive(Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
// pub struct ExtensionTest(u32);
// #[allow(non_upper_case_globals)]
// impl ExtensionTest {
//     pub const PositionTracking: Self = Self(1);
//
//     pub const fn name_for_supported(&self) -> Option<&'static str> {
//         let name = match *self {
//             Self::PositionTracking => "trc_position_tracking",
//             _ => return None,
//         };
//         Some(name)
//     }
//     pub const fn from_raw(raw: u32) -> Self {
//         Self(raw)
//     }
//     pub const fn as_raw(&self) -> u32 {
//         self.0
//     }
// }
//
// impl Display for ExtensionTest {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self.name_for_supported() {
//             Some(name) => f.write_str(name)?,
//             None => f.write_str(&format!("Unkown({})", self.as_raw()))?,
//         };
//         Ok(())
//     }
// }

self::internal::generate_extensions!(
    ClientExtensions,
    C2SPacketExtensions,
    S2CPacketExtensions,
    [
        PositionTracking,
        "trc_position_tracking",
        RcC2SPacket,
        RcS2CPacket
    ] // [Pathfinding, "trc_pathfinding"]
);

// This Probably needs some form of Unknown(ext)
self::internal::generate_extensions!(
    TurtleExtensions,
    T2SPacketExtensions,
    S2TPacketExtensions,
    [
        PositionTracking,
        "trc_position_tracking",
        RcT2SPacket,
        RcS2TPacket
    ] // [Pathfinding, "trc_pathfinding"]
);

mod internal {
    macro_rules! generate_extensions {
    ($enum_name: ident, $packet_2_server_enum_name: ident, $packet_from_server_enum_name: ident, $([$varient: ident, $name: literal, $packet_2_server_enum: ident, $packet_from_server_enum: ident]),*) => {
        #[derive(serde::Serialize,serde::Deserialize, Clone, Copy, Debug)]
        pub enum $enum_name {
        $(
            #[serde(rename = $name)]
            $varient,
        )*
        }

        impl $enum_name {
            pub const fn string_ident(&self) -> &'static str {
                match self {
                    $(
                    Self::$varient => $name,
                    )*
                }
            }
        }

        #[derive(serde::Serialize,serde::Deserialize, Clone, Debug)]
        pub enum $packet_2_server_enum_name {
        $(
            #[serde(rename = $name)]
            $varient($packet_2_server_enum),
        )*
        }

        #[derive(serde::Serialize,serde::Deserialize, Clone, Debug)]
        pub enum $packet_from_server_enum_name {
        $(
            #[serde(rename = $name)]
            $varient($packet_from_server_enum),
        )*
        }
    };
}
    pub(super) use generate_extensions;
}
