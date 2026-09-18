#!/bin/bash
# Install git hooks
echo "Installing git pre-commit hook..."

cat << 'HOOK' > .git/hooks/pre-commit
#!/bin/bash
echo "Running pre-commit hook (Agent 13 Inspector)..."
./scripts/agent13_inspect.sh
if (( $? != 0 )); then
    echo "Pre-commit checks failed! Fix issues before committing."
    exit 1
fi
exit 0
HOOK

chmod +x .git/hooks/pre-commit
echo "Git pre-commit hook installed successfully!"
