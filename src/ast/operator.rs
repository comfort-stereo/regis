use super::grammar::GrammarRule;

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Ncl,
    Mul,
    Div,
    Add,
    Sub,
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    Neq,
    And,
    Or,
}

impl BinaryOperator {
    pub fn from_rule(rule: &GrammarRule) -> Self {
        match rule {
            GrammarRule::operator_binary_ncl => BinaryOperator::Ncl,
            GrammarRule::operator_binary_mul => BinaryOperator::Mul,
            GrammarRule::operator_binary_div => BinaryOperator::Div,
            GrammarRule::operator_binary_add => BinaryOperator::Add,
            GrammarRule::operator_binary_sub => BinaryOperator::Sub,
            GrammarRule::operator_binary_gt => BinaryOperator::Gt,
            GrammarRule::operator_binary_lt => BinaryOperator::Lt,
            GrammarRule::operator_binary_gte => BinaryOperator::Gte,
            GrammarRule::operator_binary_lte => BinaryOperator::Lte,
            GrammarRule::operator_binary_eq => BinaryOperator::Eq,
            GrammarRule::operator_binary_neq => BinaryOperator::Neq,
            GrammarRule::operator_binary_and => BinaryOperator::And,
            GrammarRule::operator_binary_or => BinaryOperator::Or,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignmentOperator {
    Direct,
    Mul,
    Div,
    Add,
    Sub,
    And,
    Or,
    Ncl,
}

impl AssignmentOperator {
    pub fn from_rule(rule: &GrammarRule) -> Self {
        match rule {
            GrammarRule::operator_assign_direct => AssignmentOperator::Direct,
            GrammarRule::operator_assign_ncl => AssignmentOperator::Ncl,
            GrammarRule::operator_assign_mul => AssignmentOperator::Mul,
            GrammarRule::operator_assign_div => AssignmentOperator::Div,
            GrammarRule::operator_assign_add => AssignmentOperator::Add,
            GrammarRule::operator_assign_sub => AssignmentOperator::Sub,
            GrammarRule::operator_assign_and => AssignmentOperator::And,
            GrammarRule::operator_assign_or => AssignmentOperator::Or,
            _ => unreachable!(),
        }
    }
}
