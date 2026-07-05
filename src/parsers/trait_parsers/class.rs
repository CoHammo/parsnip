#[macro_export]
macro_rules! parser {
    (
        $name:ident $func_name:ident
        { $( $fields:tt )* }
        { $( $init:tt )* }
    ) => {
        parser!(@munch
            { $( $fields )* }
            $name $func_name { }
            newargs()
            { $( $init )* }
            defs()
        );
    };

    (@munch
        { $field:ident: $ft:ty, $( $left:tt )* }
        $name:ident $func_name:ident { $( $fields:tt )* }
        newargs( $( $args:tt )* )
        { $( $init:tt )* }
        defs( $( $defs:tt )* )
    ) => {
        parser!(@munch
            { $( $left )* }
            $name $func_name { $( $fields )* pub $field: $ft, }
            newargs( $( $args )* $field: $ft, )
            { $( $init )* }
            defs( $( $defs )* $field, )
        );
    };

    (@munch
        { => $field:ident: $ft:ty, $( $left:tt )* }
        $name:ident $func_name:ident { $( $fields:tt )* }
        newargs( $( $args:tt )* )
        { $( $init:tt )* }
        defs( $( $defs:tt )* )
    ) => {
        parser!(@munch
            { $( $left )* }
            $name $func_name { $( $fields )* pub $field: $ft, }
            newargs( $( $args )* )
            {
                let $field: $ft;
                $( $init )*
            }
            defs( $( $defs )* $field, )
        );
    };

    (@munch
        { $arg:ident: $at:ty => $field:ident: $ft:ty, $( $left:tt )* }
        $name:ident $func_name:ident { $( $fields:tt )* }
        newargs( $( $args:tt )* )
        { $( $init:tt )* }
        defs( $( $defs:tt )* )
    ) => {
        parser!(@munch
            { $( $left )* }
            $name $func_name { $( $fields )* pub $field: $ft, }
            newargs( $( $args )* $arg: $at, )
            {
                let $field: $ft;
                $( $init )*
            }
            defs( $( $defs )* $field, )
        );
    };

    (@munch
        { $field:ident: $ft:ty = $default:expr, $( $left:tt )* }
        $name:ident $func_name:ident { $( $fields:tt )* }
        newargs( $( $args:tt )* )
        { $( $init:tt )* }
        defs( $( $defs:tt )* )
    ) => {
        parser!(@munch
            { $( $left )* }
            $name $func_name { $( $fields )* pub $field: $ft, }
            newargs( $( $args )* )
            { $( $init )* }
            defs( $( $defs )* $field: $default, )
        );
    };

    (@munch
        { }
        $name:ident $func_name:ident { $( $fields:tt )* }
        newargs( $( $arg:ident: $at:ty, )* )
        { $( $init:tt )* }
        defs( $( $defs:tt )* )
    ) => {
        #[derive(Debug)]
        pub struct $name {
            pub base: BaseParser,
            $( $fields )*
        }

        impl $name {
            pub fn new( $( $arg: $at, )* ) -> Self {
                $( $init )*
                Self {
                    base: BaseParser::new(),
                    $( $defs )*
                }
            }
        }

        pub fn $func_name($( $arg: $at, )* ) -> Parser {
            Parser::$name($name::new($( $arg, )*))
        }
    };
}

#[macro_export]
macro_rules! class {
    ($vis:vis $class:ident$( < $( $gens:tt ),* > )? {
        $( $fvis:vis $field:ident: $ty:ty ),* $(,)?
    }) => {};
}
