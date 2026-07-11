fn main() {
    println!("=== Coding Agent Harness Demos ===\n");

    println!("--- Demo 1: Guardrail Blocking ---");
    coding_agent_harness::demos::demo_guardrail_blocks();
    println!();

    println!("--- Demo 2: Feedback Loop ---");
    coding_agent_harness::demos::demo_feedback_loop();
    println!();

    println!("--- Demo 3: Plugin Extension ---");
    coding_agent_harness::demos::demo_plugin_extension();
    println!();
}