pub mod config;
pub mod error;
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_project_structure() {
        // Test that all modules are accessible
        assert!(true);
    }
    
    #[test]
    fn test_value_objects() {
        use domain::value_objects::*;
        
        let user_id = UserId::new();
        let tenant_id = TenantId::new();
        let flow_id = FlowId::new();
        
        assert_ne!(user_id.0, tenant_id.0);
        assert_ne!(flow_id.0, user_id.0);
    }
    
    #[test]
    fn test_version_object() {
        use domain::value_objects::Version;
        
        let v1 = Version::new();
        let v2 = v1.next();
        
        assert_eq!(v1.0, 1);
        assert_eq!(v2.0, 2);
        assert!(v2 > v1);
    }
    
    #[test]
    fn test_name_validation() {
        use domain::value_objects::*;
        
        let valid_name = FlowName::new("Test Flow".to_string());
        assert!(valid_name.is_ok());
        
        let empty_name = FlowName::new("".to_string());
        assert!(empty_name.is_err());
        
        let long_name = FlowName::new("a".repeat(300));
        assert!(long_name.is_err());
    }
}