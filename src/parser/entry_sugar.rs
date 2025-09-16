use crate::parser::sugar_gate;
use crate::parser::{NyashParser, ParseError};
use crate::syntax::sugar_config::{SugarConfig, SugarLevel};

/// Parse code and apply sugar based on a provided level (None/Basic/Full)
pub fn parse_with_sugar_level(
    code: &str,
    level: SugarLevel,
) -> Result<crate::ast::ASTNode, ParseError> {
    match level {
        SugarLevel::None => {
            let ast = NyashParser::parse_from_string(code)?;
            Ok(ast)
        }
        SugarLevel::Basic | SugarLevel::Full => sugar_gate::with_enabled(|| {
            let ast = NyashParser::parse_from_string(code)?;
            let cfg = SugarConfig { level };
            let ast = crate::parser::sugar::apply_sugar(ast, &cfg);
            Ok(ast)
        }),
    }
}
