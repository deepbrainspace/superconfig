#!/bin/bash
set -e

# View test coverage HTML report in browser
# Usage: ./view-coverage.sh

COVERAGE_REPORT="target/coverage/tarpaulin-report.html"

if [ ! -f "$COVERAGE_REPORT" ]; then
    echo "❌ No coverage report found. Run 'cargo tarpaulin --out html --output-dir target/coverage' first."
    exit 1
fi

echo "🧪 Opening test coverage report"
echo "📁 Report location: $COVERAGE_REPORT"

# Try different browsers (WSL-friendly)
if command -v wslview &> /dev/null; then
    # WSL environment
    wslview "$COVERAGE_REPORT"
elif command -v xdg-open &> /dev/null; then
    # Linux with desktop environment
    xdg-open "$COVERAGE_REPORT"
elif command -v open &> /dev/null; then
    # macOS
    open "$COVERAGE_REPORT"
else
    echo "🧪 Coverage report available at:"
    echo "   file://$(pwd)/$COVERAGE_REPORT"
    echo ""
    echo "💡 Copy this path to your browser to view the report"
fi

echo ""
echo "📊 Coverage features:"
echo "   • Line-by-line coverage highlighting"  
echo "   • File-by-file coverage percentages"
echo "   • Interactive source code browsing"
echo "   • Uncovered code identification"