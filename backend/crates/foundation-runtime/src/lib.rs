use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub id: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDefinition {
    pub id: String,
    pub version: u32,
    pub expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDefinition {
    pub id: String,
    pub tenant_id: Uuid,
    pub allowed_rule_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub states: Vec<String>,
    pub transitions: Vec<WorkflowTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransition {
    pub from: String,
    pub to: String,
    pub event: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: Uuid,
    pub workflow_id: String,
    pub aggregate_id: Uuid,
    pub current_state: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub id: Uuid,
    pub topic: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
pub struct FoundationRuntime {
    modules: Arc<RwLock<HashMap<String, ModuleManifest>>>,
    rules: Arc<RwLock<HashMap<String, RuleDefinition>>>,
    policies: Arc<RwLock<HashMap<String, PolicyDefinition>>>,
    workflows: Arc<RwLock<HashMap<String, WorkflowDefinition>>>,
    workflow_instances: Arc<RwLock<HashMap<Uuid, WorkflowInstance>>>,
    event_log: Arc<RwLock<Vec<RuntimeEvent>>>,
}

impl FoundationRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_module(&self, manifest: ModuleManifest) {
        self.modules
            .write()
            .expect("modules lock")
            .insert(manifest.id.clone(), manifest);
    }

    pub fn register_rule(&self, rule: RuleDefinition) {
        self.rules
            .write()
            .expect("rules lock")
            .insert(rule.id.clone(), rule);
    }

    pub fn register_policy(&self, policy: PolicyDefinition) {
        self.policies
            .write()
            .expect("policies lock")
            .insert(policy.id.clone(), policy);
    }

    pub fn register_workflow(&self, workflow: WorkflowDefinition) {
        self.workflows
            .write()
            .expect("workflows lock")
            .insert(workflow.id.clone(), workflow);
    }

    pub fn start_workflow_instance(
        &self,
        workflow_id: &str,
        aggregate_id: Uuid,
    ) -> Option<WorkflowInstance> {
        let workflow = self
            .workflows
            .read()
            .expect("workflows lock")
            .get(workflow_id)
            .cloned()?;
        let initial_state = workflow.states.first()?.clone();
        let instance = WorkflowInstance {
            id: Uuid::now_v7(),
            workflow_id: workflow_id.to_string(),
            aggregate_id,
            current_state: initial_state,
            updated_at: Utc::now(),
        };
        self.workflow_instances
            .write()
            .expect("instances lock")
            .insert(instance.id, instance.clone());
        Some(instance)
    }

    pub fn transition_workflow_instance(
        &self,
        instance_id: Uuid,
        event: &str,
    ) -> Option<WorkflowInstance> {
        let mut instances = self.workflow_instances.write().expect("instances lock");
        let instance = instances.get_mut(&instance_id)?;
        let workflows = self.workflows.read().expect("workflows lock");
        let workflow = workflows.get(&instance.workflow_id)?;
        let transition = workflow
            .transitions
            .iter()
            .find(|t| t.from == instance.current_state && t.event == event)?;
        instance.current_state = transition.to.clone();
        instance.updated_at = Utc::now();
        Some(instance.clone())
    }

    pub fn publish_event(&self, topic: impl Into<String>, payload: serde_json::Value) -> Uuid {
        let event = RuntimeEvent {
            id: Uuid::now_v7(),
            topic: topic.into(),
            payload,
            created_at: Utc::now(),
        };
        self.event_log
            .write()
            .expect("event log lock")
            .push(event.clone());
        event.id
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        RuntimeSnapshot {
            modules: self
                .modules
                .read()
                .expect("modules lock")
                .values()
                .cloned()
                .collect(),
            rules: self
                .rules
                .read()
                .expect("rules lock")
                .values()
                .cloned()
                .collect(),
            policies: self
                .policies
                .read()
                .expect("policies lock")
                .values()
                .cloned()
                .collect(),
            workflows: self
                .workflows
                .read()
                .expect("workflows lock")
                .values()
                .cloned()
                .collect(),
            workflow_instances: self
                .workflow_instances
                .read()
                .expect("instances lock")
                .values()
                .cloned()
                .collect(),
            event_log_size: self.event_log.read().expect("event log lock").len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSnapshot {
    pub modules: Vec<ModuleManifest>,
    pub rules: Vec<RuleDefinition>,
    pub policies: Vec<PolicyDefinition>,
    pub workflows: Vec<WorkflowDefinition>,
    pub workflow_instances: Vec<WorkflowInstance>,
    pub event_log_size: usize,
}
