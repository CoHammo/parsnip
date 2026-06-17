/// Macro for generating a parser struct with the given name and fields.
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
macro_rules! freshen {
    ($self:ident, $ch:expr $(, $more:expr )?) => {
        if $self.base.fresh {
            $self.base.start_byte = $ch.byte;
            $( $more )?
            $self.base.fresh = false;
        }
    };
}

/// Macro for implementing the parser methods for a given set of parser variants.
#[macro_export]
macro_rules! parser_enum {
    ($($variant:ident),*) => {
        #[derive(Debug, Default)]
        pub enum Parser {
            #[default]
            Default,
            $($variant($variant)),*
        }

        impl Parser {
            fn fresh(&self) -> bool {
                match self {
                    Self::Default => false,
                    $(Self::$variant(p) => p.base.fresh,)*
                }
            }

            fn start_byte(&self) -> usize {
                match self {
                    Self::Default => 0,
                    $(Self::$variant(p) => p.base.start_byte,)*
                }
            }

            fn stat(&self) -> Stat {
                match self {
                    Self::Default => Stat::Failed,
                    $(Self::$variant(p) => p.base.stat,)*
                }
            }

            pub fn string(&self) -> String {
                match self {
                    Self::Default => "Parser()".to_string(),
                    $(Self::$variant(p) => p.string(),)*
                }
            }

            pub fn parse(&mut self, text: &Text, range: impl RangeBounds<usize>) -> Stat {
                match self {
                    Self::Default => Stat::Failed,
                    $(Self::$variant(p) => {
                        let mut chars = text.chars(range);
                        while chars.next() {
                            match p.take_char(&mut chars) {
                                Stat::Matched(_) | Stat::Failed => break,
                                _ => {},
                            }
                        }
                        return match p.base.stat {
                            Stat::Running | Stat::PossibleMatch(_) => p.finish(&chars.char),
                            _ => p.base.stat,
                        }
                    },)*
                }
            }

            pub fn take_tokens(&mut self) -> Option<Vec<Token>> {
                match self {
                    Self::Default => None,
                    $(Self::$variant(p) => p.base.tokens.take(),)*
                }
            }

            fn take_char(&mut self, chars: &mut ParseChars) -> Stat {
                match self {
                    Self::Default => Stat::Failed,
                    $(Self::$variant(p) => p.take_char(chars),)*
                }
            }

            fn finish(&mut self, ch: &Char) -> Stat {
                match self {
                    Self::Default => Stat::Failed,
                    $(Self::$variant(p) => p.finish(ch),)*
                }
            }

            fn reset(&mut self) {
                match self {
                    Self::Default => {},
                    $(Self::$variant(p) => p.reset(),)*
                }
            }

            pub fn tokens_str(&mut self) -> String {
                match self {
                    Self::Default => "".to_string(),
                    $(Self::$variant(p) => format!("{:#?}", p.base.tokens),)*
                }
            }
        }

        impl Clone for Parser {
            fn clone(&self) -> Self {
                match self {
                    Self::Default => Self::Default,
                    $(Self::$variant(p) => Self::$variant(p.clone()),)*
                }
            }
        }
    };
}
