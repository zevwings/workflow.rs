//! 服务生命周期作用域

/// 服务生命周期作用域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Scope {
    /// 单例：整个应用生命周期中只有一个实例
    #[default]
    Singleton,
    /// 瞬态：每次请求都创建新实例
    Transient,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_scope_default() {
        assert_eq!(Scope::default(), Scope::Singleton);
    }

    #[test]
    fn test_scope_traits() {
        // 测试 Clone, Copy, Debug, PartialEq, Eq
        let scope1 = Scope::Singleton;
        let scope2 = scope1; // Copy
        let scope3 = scope1; // Clone
        let debug_str = format!("{:?}", scope1);

        assert_eq!(scope1, scope2);
        assert_eq!(scope1, scope3);
        assert_eq!(debug_str, "Singleton");
    }

    #[test]
    fn test_scope_equality() {
        // 测试相等性和不等性
        assert_eq!(Scope::Singleton, Scope::Singleton);
        assert_eq!(Scope::Transient, Scope::Transient);
        assert_ne!(Scope::Singleton, Scope::Transient);
        assert_ne!(Scope::Transient, Scope::Singleton);
    }
}
