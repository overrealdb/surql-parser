/// A shorthand for token kinds.
macro_rules! t {
	("invalid") => {
		$crate::upstream::syn::token::TokenKind::Invalid
	};
	("eof") => {
		$crate::upstream::syn::token::TokenKind::Eof
	};
	("[") => {
		$crate::upstream::syn::token::TokenKind::OpenDelim(
			$crate::upstream::syn::token::Delim::Bracket,
		)
	};
	("{") => {
		$crate::upstream::syn::token::TokenKind::OpenDelim(
			$crate::upstream::syn::token::Delim::Brace,
		)
	};
	("(") => {
		$crate::upstream::syn::token::TokenKind::OpenDelim(
			$crate::upstream::syn::token::Delim::Paren,
		)
	};
	("]") => {
		$crate::upstream::syn::token::TokenKind::CloseDelim(
			$crate::upstream::syn::token::Delim::Bracket,
		)
	};
	("}") => {
		$crate::upstream::syn::token::TokenKind::CloseDelim(
			$crate::upstream::syn::token::Delim::Brace,
		)
	};
	(")") => {
		$crate::upstream::syn::token::TokenKind::CloseDelim(
			$crate::upstream::syn::token::Delim::Paren,
		)
	};
	("r\"") => {
		$crate::upstream::syn::token::TokenKind::String(
			$crate::upstream::syn::token::StringKind::RecordIdDouble,
		)
	};
	("r'") => {
		$crate::upstream::syn::token::TokenKind::String(
			$crate::upstream::syn::token::StringKind::RecordId,
		)
	};
	("u\"") => {
		$crate::upstream::syn::token::TokenKind::String(
			$crate::upstream::syn::token::StringKind::UuidDouble,
		)
	};
	("u'") => {
		$crate::upstream::syn::token::TokenKind::String(
			$crate::upstream::syn::token::StringKind::Uuid,
		)
	};
	("d\"") => {
		$crate::upstream::syn::token::TokenKind::String(
			$crate::upstream::syn::token::StringKind::DateTimeDouble,
		)
	};
	("d'") => {
		$crate::upstream::syn::token::TokenKind::String(
			$crate::upstream::syn::token::StringKind::DateTime,
		)
	};
	("b\"") => {
		$crate::upstream::syn::token::TokenKind::String(
			$crate::upstream::syn::token::StringKind::BytesDouble,
		)
	};
	("b'") => {
		$crate::upstream::syn::token::TokenKind::String(
			$crate::upstream::syn::token::StringKind::Bytes,
		)
	};
	("f\"") => {
		$crate::upstream::syn::token::TokenKind::String(
			$crate::upstream::syn::token::StringKind::FileDouble,
		)
	};
	("f'") => {
		$crate::upstream::syn::token::TokenKind::String(
			$crate::upstream::syn::token::StringKind::File,
		)
	};
	("\"") => {
		$crate::upstream::syn::token::TokenKind::String(
			$crate::upstream::syn::token::StringKind::PlainDouble,
		)
	};
	("'") => {
		$crate::upstream::syn::token::TokenKind::String(
			$crate::upstream::syn::token::StringKind::Plain,
		)
	};
	("\"r") => {
		$crate::upstream::syn::token::TokenKind::CloseString { double: true }
	};
	("'r") => {
		$crate::upstream::syn::token::TokenKind::CloseString { double: false }
	};
	("f") => {
		$crate::upstream::syn::token::TokenKind::NumberSuffix(
			$crate::upstream::syn::token::NumberSuffix::Float,
		)
	};
	("dec") => {
		$crate::upstream::syn::token::TokenKind::NumberSuffix(
			$crate::upstream::syn::token::NumberSuffix::Decimal,
		)
	};
	("<") => {
		$crate::upstream::syn::token::TokenKind::LeftChefron
	};
	(">") => {
		$crate::upstream::syn::token::TokenKind::RightChefron
	};
	("<|") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::KnnOpen,
		)
	};
	(";") => {
		$crate::upstream::syn::token::TokenKind::SemiColon
	};
	(",") => {
		$crate::upstream::syn::token::TokenKind::Comma
	};
	("|") => {
		$crate::upstream::syn::token::TokenKind::Vert
	};
	("...") => {
		$crate::upstream::syn::token::TokenKind::DotDotDot
	};
	("..") => {
		$crate::upstream::syn::token::TokenKind::DotDot
	};
	(".") => {
		$crate::upstream::syn::token::TokenKind::Dot
	};
	("::") => {
		$crate::upstream::syn::token::TokenKind::PathSeperator
	};
	(":") => {
		$crate::upstream::syn::token::TokenKind::Colon
	};
	("->") => {
		$crate::upstream::syn::token::TokenKind::ArrowRight
	};
	("*") => {
		$crate::upstream::syn::token::TokenKind::Star
	};
	("$") => {
		$crate::upstream::syn::token::TokenKind::Dollar
	};
	("+") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Add,
		)
	};
	("%") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Modulo,
		)
	};
	("-") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Subtract,
		)
	};
	("**") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Power,
		)
	};
	("*=") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::AllEqual,
		)
	};
	("*~") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::AllLike,
		)
	};
	("/") => {
		$crate::upstream::syn::token::TokenKind::ForwardSlash
	};
	("<=") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::LessEqual,
		)
	};
	(">=") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::GreaterEqual,
		)
	};
	("@") => {
		$crate::upstream::syn::token::TokenKind::At
	};
	("||") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Or,
		)
	};
	("&&") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::And,
		)
	};
	("×") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Mult,
		)
	};
	("÷") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Divide,
		)
	};
	("$param") => {
		$crate::upstream::syn::token::TokenKind::Parameter
	};
	("!") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Not,
		)
	};
	("!~") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::NotLike,
		)
	};
	("!=") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::NotEqual,
		)
	};
	("?") => {
		$crate::upstream::syn::token::TokenKind::Question
	};
	("?:") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Tco,
		)
	};
	("==") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Exact,
		)
	};
	("!=") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::NotEqual,
		)
	};
	("*=") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::AllEqual,
		)
	};
	("?=") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::AnyEqual,
		)
	};
	("=") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Equal,
		)
	};
	("!~") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::NotLike,
		)
	};
	("*~") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::AllLike,
		)
	};
	("?~") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::AnyLike,
		)
	};
	("~") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Like,
		)
	};
	("+?=") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Ext,
		)
	};
	("+=") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Inc,
		)
	};
	("-=") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Dec,
		)
	};
	("∋") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Contains,
		)
	};
	("∌") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::NotContains,
		)
	};
	("∈") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::Inside,
		)
	};
	("∉") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::NotInside,
		)
	};
	("⊇") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::ContainsAll,
		)
	};
	("⊃") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::ContainsAny,
		)
	};
	("⊅") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::ContainsNone,
		)
	};
	("⊆") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::AllInside,
		)
	};
	("⊂") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::AnyInside,
		)
	};
	("⊄") => {
		$crate::upstream::syn::token::TokenKind::Operator(
			$crate::upstream::syn::token::Operator::NoneInside,
		)
	};
	("EDDSA") => {
		$crate::upstream::syn::token::TokenKind::Algorithm($crate::upstream::sql::Algorithm::EdDSA)
	};
	("ES256") => {
		$crate::upstream::syn::token::TokenKind::Algorithm($crate::upstream::sql::Algorithm::Es256)
	};
	("ES384") => {
		$crate::upstream::syn::token::TokenKind::Algorithm($crate::upstream::sql::Algorithm::Es384)
	};
	("ES512") => {
		$crate::upstream::syn::token::TokenKind::Algorithm($crate::upstream::sql::Algorithm::Es512)
	};
	("HS256") => {
		$crate::upstream::syn::token::TokenKind::Algorithm($crate::upstream::sql::Algorithm::Hs256)
	};
	("HS384") => {
		$crate::upstream::syn::token::TokenKind::Algorithm($crate::upstream::sql::Algorithm::Hs384)
	};
	("HS512") => {
		$crate::upstream::syn::token::TokenKind::Algorithm($crate::upstream::sql::Algorithm::Hs512)
	};
	("PS256") => {
		$crate::upstream::syn::token::TokenKind::Algorithm($crate::upstream::sql::Algorithm::Ps256)
	};
	("PS384") => {
		$crate::upstream::syn::token::TokenKind::Algorithm($crate::upstream::sql::Algorithm::Ps384)
	};
	("PS512") => {
		$crate::upstream::syn::token::TokenKind::Algorithm($crate::upstream::sql::Algorithm::Ps512)
	};
	("RS256") => {
		$crate::upstream::syn::token::TokenKind::Algorithm($crate::upstream::sql::Algorithm::Rs256)
	};
	("RS384") => {
		$crate::upstream::syn::token::TokenKind::Algorithm($crate::upstream::sql::Algorithm::Rs384)
	};
	("RS512") => {
		$crate::upstream::syn::token::TokenKind::Algorithm($crate::upstream::sql::Algorithm::Rs512)
	};
	("CHEBYSHEV") => {
		$crate::upstream::syn::token::TokenKind::Distance(
			$crate::upstream::syn::token::DistanceKind::Chebyshev,
		)
	};
	("COSINE") => {
		$crate::upstream::syn::token::TokenKind::Distance(
			$crate::upstream::syn::token::DistanceKind::Cosine,
		)
	};
	("EUCLIDEAN") => {
		$crate::upstream::syn::token::TokenKind::Distance(
			$crate::upstream::syn::token::DistanceKind::Euclidean,
		)
	};
	("HAMMING") => {
		$crate::upstream::syn::token::TokenKind::Distance(
			$crate::upstream::syn::token::DistanceKind::Hamming,
		)
	};
	("JACCARD") => {
		$crate::upstream::syn::token::TokenKind::Distance(
			$crate::upstream::syn::token::DistanceKind::Jaccard,
		)
	};
	("MANHATTAN") => {
		$crate::upstream::syn::token::TokenKind::Distance(
			$crate::upstream::syn::token::DistanceKind::Manhattan,
		)
	};
	("MAHALANOBIS") => {
		$crate::upstream::syn::token::TokenKind::Distance(
			$crate::upstream::syn::token::DistanceKind::Mahalanobis,
		)
	};
	("MINKOWSKI") => {
		$crate::upstream::syn::token::TokenKind::Distance(
			$crate::upstream::syn::token::DistanceKind::Minkowski,
		)
	};
	("PEARSON") => {
		$crate::upstream::syn::token::TokenKind::Distance(
			$crate::upstream::syn::token::DistanceKind::Pearson,
		)
	};
	("F64") => {
		$crate::upstream::syn::token::TokenKind::VectorType(
			$crate::upstream::syn::token::VectorTypeKind::F64,
		)
	};
	("F32") => {
		$crate::upstream::syn::token::TokenKind::VectorType(
			$crate::upstream::syn::token::VectorTypeKind::F32,
		)
	};
	("I64") => {
		$crate::upstream::syn::token::TokenKind::VectorType(
			$crate::upstream::syn::token::VectorTypeKind::I64,
		)
	};
	("I32") => {
		$crate::upstream::syn::token::TokenKind::VectorType(
			$crate::upstream::syn::token::VectorTypeKind::I32,
		)
	};
	("I16") => {
		$crate::upstream::syn::token::TokenKind::VectorType(
			$crate::upstream::syn::token::VectorTypeKind::I16,
		)
	};
	($t:tt) => {
		$crate::upstream::syn::token::TokenKind::Keyword($crate::upstream::syn::token::keyword_t!(
			$t
		))
	};
}
pub(crate) use t;
