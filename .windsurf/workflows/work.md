# Browser Project Parallel Development Workflow (Solo Dev - Worktrees)

This workflow orchestrates parallel development using git worktrees, allowing multiple subagents to work independently on the same repository, then merge completed features directly to main.

## Phase 1: Plan

### 1.1 Analyze Current State

- Read `AGENTS.md` to understand project rules and architecture
- Read `INTERFACES.md` to understand subsystem dependencies and priorities
- Read `README.md` to understand project goals
- Examine current codebase structure in `src/`
- Check git status to ensure clean working directory

### 1.2 Define Priority Features

Analyze the project state and define features that move the project toward a fully working browser:

1. Read `INTERFACES.md` to understand which subsystems exist vs. "NOT YET BUILT"
2. Examine current implementation in `src/` to assess completion level
3. Review recent git commits to understand what's been recently implemented
4. Identify the highest-impact subsystems that can be built in parallel
5. Create features following the dependency chain (foundation → rendering → features)
6. Each feature should be completable in ~200-500 lines of Rust per AGENTS.md guidelines
7. Ensure features respect the parallel agent workflow - independent tasks should be in the same priority tier
8. Document each feature with: worktree name, description, dependencies, and acceptance criteria

### 1.3 Risk Assessment

- **Worktree conflicts**: Ensure each worktree has unique name
- **Subagent coordination**: Each agent works independently, no shared state
- **Resource limits**: Monitor system resources with parallel agents
- **Code quality**: Each agent must run tests, clippy, fmt before merge

### 1.4 Success Criteria

- All worktrees created successfully
- All subagents launched and working independently
- All features pass quality checks
- Features merged to main successfully
- Worktrees cleaned up properly

## Phase 2: Execute

### 2.1 Preparation

```bash
# Ensure clean git state
git status
git pull origin main

# Create worktrees directory
mkdir -p .worktrees
```

### 2.2 Create Worktrees

For each feature in the plan:

```bash
# Worktree naming pattern: .worktrees/feature-name
git worktree add .worktrees/style-engine -b feature/style-engine
git worktree add .worktrees/layout-block -b feature/layout-block
# ... repeat for all features
```

### 2.3 Launch Subagents

Use `run_subagent` with `is_background=true` for parallel execution. Each agent gets:

#### Agent Template

```text
You are implementing [FEATURE_NAME] for a Rust browser built from scratch.

Context:
- Read INTERFACES.md for the trait you must satisfy
- Read AGENTS.md for project rules
- Your worktree is at: .worktrees/[feature-name]
- Your branch is: feature/[feature-name]


Task:
1. Implement the [FEATURE_NAME] subsystem
2. Write comprehensive tests
3. Run: cargo test --all-features
4. Run: cargo clippy --all-targets --all-features -- -D warnings
5. Run: cargo fmt --all -- --check
6. Run: cargo build --release --all-features
7. Commit changes with descriptive message
8. Do NOT push - the main agent will handle merging


Constraints:
- Work only in your assigned worktree
- Do not modify other subsystems
- Follow existing code patterns and style
- All tests must pass before completion

```

### 2.4 Monitor Subagent Progress

```bash
# Check worktree statuses
git worktree list

# Monitor each agent periodically using read_subagent
# Track completion status

```

### 2.5 Merge Completed Features

For each completed worktree:

```bash
# Switch to main
git checkout main
git pull origin main

# Merge the feature branch
git merge feature/[feature-name]

# Run final verification
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo build --release --all-features

# If all checks pass, push to main
git push origin main

# Delete the merged branch
git branch -d feature/[feature-name]
```

### 2.6 Cleanup

```bash
# Remove worktrees
git worktree remove .worktrees/style-engine
git worktree remove .worktrees/layout-block
# ... repeat for all worktrees

# Delete all merged feature branches
git branch -d feature/style-engine
git branch -d feature/layout-block
# ... repeat for all merged branches
```

## Phase 3: Reflect

### 3.1 What Worked

- Document which subsystems were implemented successfully
- Note any patterns that emerged across implementations
- Identify which agents completed fastest/slowest

### 3.2 What Failed

- Track any subagents that timed out or failed
- Note any merge conflicts or issues
- Identify features that failed quality checks

### 3.3 Process Improvements

- Was the number of parallel agents optimal?
- Were the feature definitions adequate?
- Did the worktree isolation work as expected?
- Should we adjust the agent template?

### 3.4 Technical Learnings

- Any new patterns discovered in the codebase
- Architectural insights from implementing multiple subsystems
- Dependencies that were different than expected

## Phase 4: Document

### 4.1 Update AGENTS.md

Add learnings only if they provide durable guidance:

#### If successful
- Document the optimal number of parallel agents for this project
- Add any new architectural patterns discovered
- Update the parallel agent workflow section

#### If issues found
- Update branch management rules if conflicts occurred
- Add troubleshooting steps for subagent failures
- Document any new testing requirements

### 4.2 Update INTERFACES.md

- Mark implemented subsystems as "EXISTS" instead of "NOT YET BUILT"
- Update any interface definitions that changed during implementation
- Adjust priority tiers if dependencies were different than expected

### 4.3 Cleanup

- Remove .worktrees directory if empty
- Ensure no temporary files remain
- Verify git worktree list is clean

## Emergency Procedures

### If Subagent Fails

1. Check the agent output using `read_subagent`
2. If recoverable, provide guidance and resume
3. If stuck, mark as failed and continue with others
4. Document failure in reflection phase

### If Merge Conflicts

```bash
# Resolve conflicts manually
git checkout main
git merge feature/[feature-name]
# Resolve conflicts
git add .
git commit -m "Merge feature/[feature-name] with conflict resolution"
```

### If Worktree Issues

```bash
# Remove stuck worktree
git worktree remove --force .worktrees/[feature-name]

# Clean up branches
git branch -D feature/[feature-name]
```

### If System Resources Overloaded

- Reduce number of parallel agents
- Monitor with `top` or `htop`
- Consider running in batches instead of all at once

## Success Metrics

- [ ] All worktrees created without errors
- [ ] All subagents launched successfully
- [ ] Most subagents complete their tasks successfully
- [ ] All completed features pass quality checks
- [ ] Features merged to main successfully
- [ ] Worktrees cleaned up properly
- [ ] AGENTS.md updated with learnings
- [ ] INTERFACES.md updated with new status

## Notes

- This workflow is designed for solo development with parallel execution
- Adjust the number of features based on project needs and system capacity
- The workflow can be run iteratively (e.g., 5 features at a time)
- Always maintain clean git state before starting
- Merge features to main as they complete to avoid large merge conflicts
