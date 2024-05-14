// This Probably needs some form of Unknown(ext)
self::internal::generate_extensions!(
    Extensions,
    [PositionTracking, "trc_position_tracking"],
    [Pathfinding, "trc_pathfinding"]
);

mod internal {
    macro_rules! generate_extensions {
    ($enum_name: ident,$([$varient: ident, $name: literal]),*) => {
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
    };
}
    pub(super) use generate_extensions;
}
