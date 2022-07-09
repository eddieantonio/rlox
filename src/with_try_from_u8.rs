/// Derives a [TryFrom<u8>] implementation for the enum.
///
/// Macro adapted from: <https://stackoverflow.com/a/57578431/6626414>
#[macro_export]
macro_rules! with_try_from_u8 {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        // match each variant in the enum
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        // create the enum
        $(#[$meta])*
        $vis enum $name {
            // Create each variant, verbatim
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        // create the TryFrom implementation:
        impl std::convert::TryFrom<u8> for $name {
            type Error = ();

            fn try_from(v: u8) -> Result<Self, Self::Error> {
                match v {
                    // create a match arm for each variant:
                    $(x if x == $name::$vname as u8 => Ok($name::$vname),)*
                    _ => Err(()),
                }
            }
        }
    }
}
