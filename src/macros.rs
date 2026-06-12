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
            pub stat: Stat,
            pub start_byte: usize,
            pub fresh: bool,
            pub tokens: Option<Vec<Token>>,
            $( $fields )*
        }

        impl $name {
            pub fn new( $( $arg: $at, )* ) -> Self {
                $( $init )*
                Self {
                    stat: Stat::Running,
                    fresh: true,
                    start_byte: 0,
                    tokens: None,
                    $( $defs )*
                }
            }

            // pub fn new_parser( $( $arg: $at, )* ) -> Parser {
            //     Parser::new(Parser::$name($name::new($( $arg, )*)))
            // }

            pub fn add_tokens(&mut self, tokens: Option<Vec<Token>>) {
                if let Some(extra_tokens) = tokens {
                    if let Some(toks) = &mut self.tokens {
                        toks.extend(extra_tokens);
                    } else {
                        self.tokens = Some(extra_tokens);
                    }
                }
            }

            fn reset_base(&mut self) {
                self.stat = Stat::Running;
                self.start_byte = 0;
                self.tokens = None;
                self.fresh = true;
            }

            pub fn token_count(&self) -> usize {
                if let Some(toks) = &self.tokens {
                    toks.len()
                } else {
                    0
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
    ($self:ident, $ch:ident) => {
        if $self.fresh {
            $self.start_byte = $ch.byte;
            $self.fresh = false;
        }
    };
}

/// Macro for implementing the parser methods for a given set of parser variants.
#[macro_export]
macro_rules! make_parsers {
    ($($variant:ident),*) => {
        #[derive(Debug, Default, Clone)]
        pub enum Parser {
            #[default]
            Default,
            $($variant($variant)),*
        }

        impl Parser {
            fn fresh(&self) -> bool {
                match self {
                    Self::Default => false,
                    $(Self::$variant(p) => p.fresh,)*
                }
            }

            fn start_byte(&self) -> usize {
                match self {
                    Self::Default => 0,
                    $(Self::$variant(p) => p.start_byte,)*
                }
            }

            fn stat(&self) -> Stat {
                match self {
                    Self::Default => Stat::Failed,
                    $(Self::$variant(p) => p.stat,)*
                }
            }

            pub fn string(&self) -> String {
                match self {
                    Self::Default => "Parser()".to_string(),
                    $(Self::$variant(p) => p.string(),)*
                }
            }

            pub fn parse(&mut self, text: &Text, from_char: Option<usize>, to_char: Option<usize>) -> Stat {
                match self {
                    Self::Default => Stat::Failed,
                    $(Self::$variant(p) => {
                        let mut last_char = Char::empty();
                        for ch in text.chars(from_char, to_char) {
                            last_char = ch.owned();
                            match p.take_char(&ch) {
                                Stat::Matched(_) | Stat::Failed => break,
                                _ => {},
                            }
                        }
                        return match p.stat {
                            Stat::Running | Stat::HasMatch(_) => p.finish(&last_char),
                            _ => p.stat,
                        }
                    },)*
                }
            }

            pub fn take_tokens(&mut self) -> Option<Vec<Token>> {
                match self {
                    Self::Default => None,
                    $(Self::$variant(p) => p.tokens.take(),)*
                }
            }

            pub fn token_count(&mut self) -> usize {
                match self {
                    Self::Default => 0,
                    $(Self::$variant(p) => p.token_count(),)*
                }
            }

            fn take_char(&mut self, value: &Char) -> Stat {
                match self {
                    Self::Default => Stat::Failed,
                    $(Self::$variant(p) => p.take_char(value),)*
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
                    $(Self::$variant(p) => format!("{:?}", p.tokens),)*
                }
            }
        }
    };
}
