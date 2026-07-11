use crate::GuardrailAction;
use async_trait::async_trait;

#[async_trait]
pub trait HitlConfirmer: Send + Sync {
    async fn confirm(&self, action: &GuardrailAction) -> harness_core::Result<bool>;
}

pub struct MockHitlConfirmer {
    approve: bool,
}

impl MockHitlConfirmer {
    pub fn new(approve: bool) -> Self { Self { approve } }
}

#[async_trait]
impl HitlConfirmer for MockHitlConfirmer {
    async fn confirm(&self, _action: &GuardrailAction) -> harness_core::Result<bool> {
        Ok(self.approve)
    }
}

pub struct StdioHitlConfirmer;

#[async_trait]
impl HitlConfirmer for StdioHitlConfirmer {
    async fn confirm(&self, action: &GuardrailAction) -> harness_core::Result<bool> {
        let reason = match action {
            GuardrailAction::Block { reason } => reason,
            _ => return Ok(true),
        };
        eprintln!("\n[HITL] Action blocked: {}", reason);
        eprintln!("[HITL] Approve? (y/n): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        Ok(input.trim().eq_ignore_ascii_case("y"))
    }
}