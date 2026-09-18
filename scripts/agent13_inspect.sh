#!/bin/bash

# Agent 13 Inspection Script
# Performs static analysis, detects common code smells, and checks for security issues.

# Indentation: 2 spaces. Bash 5 conditionals used.
# Follows Omarchy Web Management Interface guidelines.

echo "Starting Agent 13 Code Inspection..."

EXIT_CODE=0

# 1. Check for hardcoded paths instead of environment variables
echo "Checking for hardcoded absolute paths..."
if grep -rnE "/etc/omarchy|/usr/lib/omarchy|/var/log/omarchy" --exclude-dir=.git .; then
  echo "WARNING: Hardcoded paths found! Use variables like \$OMARCHY_PATH instead."
  # Not setting EXIT_CODE=1 for warnings, but it highlights a potential code smell
fi

# 2. Check for root execution (Security Issue)
echo "Checking for root execution instructions (e.g. running daemon as root)..."
if grep -rnwi "user=root" --include="*.service" --exclude-dir=.git .; then
  echo "ERROR: Found potential root execution in service files! Daemons must drop privileges."
  EXIT_CODE=1
fi

if grep -rnwi "sudo " --exclude-dir=.git --exclude="agent13_inspect.sh" .; then
  echo "WARNING: Usage of sudo detected. Ensure polkit is used appropriately."
fi

# 3. Check for heavy frontend frameworks (Performance/Architecture Issue)
echo "Checking for prohibited frontend frameworks (React, Vue, Angular, PatternFly)..."
if grep -rnwiE "react|vue|angular|patternfly" --include="*.js" --include="*.html" --include="package.json" --exclude-dir=.git .; then
  echo "ERROR: Prohibited heavy web frameworks detected! Must use Web TUI guidelines."
  EXIT_CODE=1
fi

# 4. Enforce Bash Scripting Style (Omarchy conventions)
echo "Checking Bash script conventions..."
while IFS= read -r -d '' script; do
  # Check shebang
  if head -n 1 "$script" | grep -qv "#!/bin/bash"; then
    echo "ERROR: Incorrect shebang in $script. Must be exactly #!/bin/bash"
    EXIT_CODE=1
  fi
  # Basic check for single brackets instead of bash 5 double brackets for strings
  if grep -q " \[ " "$script"; then
     echo "WARNING: Found ' [ ' in $script. Prefer Bash 5 conditional [[ ]] for strings."
  fi
done < <(find . -type f -name "*.sh" -not -path "*/.git/*" -print0)

# 5. Run standard linters (shellcheck)
if command -v shellcheck >/dev/null 2>&1; then
  echo "Running shellcheck on bash scripts..."
  if ! find . -type f -name "*.sh" -not -path "*/.git/*" -exec shellcheck {} +; then
    echo "ERROR: Shellcheck found issues."
    EXIT_CODE=1
  fi
else
  echo "WARNING: shellcheck is not installed. Skipping."
fi

# 6. Check if PR Template exists
if [[ ! -f ".github/PULL_REQUEST_TEMPLATE.md" ]]; then
  echo "WARNING: .github/PULL_REQUEST_TEMPLATE.md is missing!"
fi

echo "Inspection complete."

if (( EXIT_CODE != 0 )); then
  echo "Agent 13 Inspection FAILED due to architecture or security violations."
  exit $EXIT_CODE
else
  echo "Agent 13 Inspection PASSED."
  exit 0
fi
