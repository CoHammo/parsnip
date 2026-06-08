/// Macro for generating a parser struct with the given name and fields.
#[macro_export]
macro_rules! parser {
    (
        $name:ident $func_name:ident
        { $( $fields:tt )* }
        ($keys:ident)
        { $( $init:tt )* }
    ) => {
        parser!(@munch
            { $( $fields )* }
            $name $func_name { }
            newargs()
            ($keys)
            { $( $init )* }
            defs()
        );
    };

    (@munch
        { $field:ident: $ft:ty, $( $left:tt )* }
        $name:ident $func_name:ident { $( $fields:tt )* }
        newargs( $( $args:tt )* )
        ($keys:ident)
        { $( $init:tt )* }
        defs( $( $defs:tt )* )
    ) => {
        parser!(@munch
            { $( $left )* }
            $name $func_name { $( $fields )* pub $field: $ft, }
            newargs( $( $args )* $field: $ft, )
            ($keys)
            { $( $init )* }
            defs( $( $defs )* $field, )
        );
    };

    (@munch
        { => $field:ident: $ft:ty, $( $left:tt )* }
        $name:ident $func_name:ident { $( $fields:tt )* }
        newargs( $( $args:tt )* )
        ($keys:ident)
        { $( $init:tt )* }
        defs( $( $defs:tt )* )
    ) => {
        parser!(@munch
            { $( $left )* }
            $name $func_name { $( $fields )* pub $field: $ft, }
            newargs( $( $args )* )
            ($keys)
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
        ($keys:ident)
        { $( $init:tt )* }
        defs( $( $defs:tt )* )
    ) => {
        parser!(@munch
            { $( $left )* }
            $name $func_name { $( $fields )* pub $field: $ft, }
            newargs( $( $args )* $arg: $at, )
            ($keys)
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
        ($keys:ident)
        { $( $init:tt )* }
        defs( $( $defs:tt )* )
    ) => {
        parser!(@munch
            { $( $left )* }
            $name $func_name { $( $fields )* pub $field: $ft, }
            newargs( $( $args )* )
            ($keys)
            { $( $init )* }
            defs( $( $defs )* $field: $default, )
        );
    };

    (@munch
        { }
        $name:ident $func_name:ident { $( $fields:tt )* }
        newargs( $( $arg:ident: $at:ty, )* )
        ($keys:ident)
        { $( $init:tt )* }
        defs( $( $defs:tt )* )
    ) => {
        #[derive(Debug)]
        pub struct $name {
            pub keys: Option<Arc<[Box<str>]>>,
            pub stat: Stat,
            pub start_byte: usize,
            pub fresh: bool,
            pub tokens: Option<Vec<Token>>,
            $( $fields )*
        }

        impl $name {
            pub fn new( $( $arg: $at, )* ) -> Self {
                let $keys: Option<Arc<[Box<str>]>>;
                $( $init )*
                Self {
                    keys: $keys,
                    stat: Stat::Running,
                    fresh: true,
                    start_byte: 0,
                    tokens: None,
                    $( $defs )*
                }
            }

            // pub fn new_parser( $( $arg: $at, )* ) -> Parser {
            //     Parser::new(ParserEnum::$name($name::new($( $arg, )*)))
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

            fn fresh_check(&mut self, byte_index: usize) {
                if self.fresh {
                    self.fresh = false;
                    self.start_byte = byte_index;
                }
            }

            fn reset_base(&mut self) {
                self.stat = Stat::Running;
                self.start_byte = 0;
                self.tokens = None;
                self.fresh = true;
            }

            fn token_count(&self) -> usize {
                if let Some(toks) = &self.tokens {
                    toks.len()
                } else {
                    0
                }
            }
        }

        #[wasm_bindgen]
        pub fn $func_name($( $arg: $at, )* ) -> Parser {
            Parser::new(ParserEnum::$name($name::new($( $arg, )*)))
        }
    };
}

/// Macro for implementing the parser methods for a given set of parser variants.
#[macro_export]
macro_rules! make_parsers {
    ($($variant:ident),*) => {
        #[derive(Debug, Default, Clone)]
        enum ParserEnum {
            #[default]
            Default,
            $($variant($variant)),*
        }

        #[derive(Debug, Clone)]
        #[wasm_bindgen]
        pub struct Parser(ParserEnum);

        #[wasm_bindgen]
        impl Parser {
            fn new(parser: ParserEnum) -> Self {
                Parser(parser)
            }

            fn keys(&self) -> Option<Arc<[Box<str>]>> {
                match &self.0 {
                    ParserEnum::Default => None,
                    $(ParserEnum::$variant(p) => p.keys.clone(),)*
                }
            }

            fn stat(&self) -> Stat {
                match &self.0 {
                    ParserEnum::Default => Stat::Failed,
                    $(ParserEnum::$variant(p) => p.stat,)*
                }
            }

            pub fn string(&self) -> String {
                match &self.0 {
                    ParserEnum::Default => "Parser()".to_string(),
                    $(ParserEnum::$variant(p) => p.string(),)*
                }
            }

            fn parse(&mut self, text: &Text) -> Stat {
                match &mut self.0 {
                    ParserEnum::Default => Stat::Failed,
                    $(ParserEnum::$variant(p) => {
                        let mut c: Char = Char::empty();
                        for ch in Text::chars(text) {
                            c = ch.renew();
                            match p.take_char(&ch) {
                                Stat::Matched(_) | Stat::Failed => break,
                                _ => {},
                            }
                        }
                        return match p.stat {
                            Stat::Running | Stat::HasMatch(_) => p.finish(&c),
                            _ => p.stat,
                        }
                    },)*
                }
            }

            pub fn parse_js(&mut self, text: JsText) -> bool {
                let t = &Text::new(text);
                match self.parse(t) {
                    Stat::Matched(_) => true,
                    _ => false,
                }
            }

            fn parse_range(&mut self, text: &Text, from_char: usize, to_char: Option<usize>) -> Stat {
                match &mut self.0 {
                    ParserEnum::Default => Stat::Failed,
                    $(ParserEnum::$variant(p) => {
                        let mut c: Char = Char::empty();
                        for ch in Text::chars_range(text, from_char, to_char) {
                            c = ch.renew();
                            match p.take_char(&ch) {
                                Stat::Matched(_) | Stat::Failed => break,
                                _ => {}
                            }
                        }
                        return match p.stat {
                            Stat::Running | Stat::HasMatch(_) => p.finish(&c),
                            _ => p.stat
                        }
                    },)*
                }
            }

            pub fn parse_range_js(&mut self, text: JsText, from_char: usize, to_char: Option<usize>) -> bool {
                let t = &Text::new(text);
                match self.parse_range(t, from_char, to_char) {
                    Stat::Matched(_) => true,
                    _ => false,
                }
            }

            fn take_tokens(&mut self) -> Option<Vec<Token>> {
                match &mut self.0 {
                    ParserEnum::Default => None,
                    $(ParserEnum::$variant(p) => p.tokens.take(),)*
                }
            }

            pub fn token_count(&mut self) -> usize {
                match &mut self.0 {
                    ParserEnum::Default => 0,
                    $(ParserEnum::$variant(p) => p.token_count(),)*
                }
            }

            fn take_char(&mut self, value: &Char) -> Stat {
                match &mut self.0 {
                    ParserEnum::Default => Stat::Failed,
                    $(ParserEnum::$variant(p) => p.take_char(value),)*
                }
            }

            fn finish(&mut self, ch: &Char) -> Stat {
                match &mut self.0 {
                    ParserEnum::Default => Stat::Failed,
                    $(ParserEnum::$variant(p) => p.finish(ch),)*
                }
            }

            fn reset(&mut self) {
                match &mut self.0 {
                    ParserEnum::Default => {},
                    $(ParserEnum::$variant(p) => p.reset(),)*
                }
            }

            pub fn tokens_str(&mut self) -> String {
                match &self.0 {
                    ParserEnum::Default => "".to_string(),
                    $(ParserEnum::$variant(p) => format!("{:?}", p.tokens),)*
                }
            }
        }
    };
}
