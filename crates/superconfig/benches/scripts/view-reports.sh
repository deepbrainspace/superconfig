#!/bin/bash
set -e

# View benchmark HTML reports in browser
# Usage: ./view-reports.sh [baseline-name]

BASELINE_NAME="${1:-current}"

if [ "$BASELINE_NAME" = "current" ]; then
    REPORT_DIR="target/criterion/report"
    if [ ! -f "$REPORT_DIR/index.html" ]; then
        echo "❌ No current benchmark reports found. Run 'cargo bench' first."
        exit 1
    fi
else
    REPORT_DIR="benches/baselines/$BASELINE_NAME/criterion/report"
    if [ ! -f "$REPORT_DIR/index.html" ]; then
        echo "❌ Baseline '$BASELINE_NAME' reports not found in $REPORT_DIR"
        echo "Available baselines:"
        ls -1 benches/baselines/ 2>/dev/null || echo "  (none)"
        exit 1
    fi
fi

echo "🌐 Opening benchmark reports: $BASELINE_NAME"
echo "📁 Report location: $REPORT_DIR/index.html"

# Try different browsers (WSL-friendly)
if command -v wslview &> /dev/null; then
    # WSL environment
    wslview "$REPORT_DIR/index.html"
elif command -v xdg-open &> /dev/null; then
    # Linux with desktop environment
    xdg-open "$REPORT_DIR/index.html"
elif command -v open &> /dev/null; then
    # macOS
    open "$REPORT_DIR/index.html"
else
    echo "📊 HTML reports available at:"
    echo "   file://$(pwd)/$REPORT_DIR/index.html"
    echo ""
    echo "💡 Copy this path to your browser to view the reports"
fi

echo ""
echo "📈 Report features:"
echo "   • Interactive charts and graphs"
echo "   • Performance regression analysis"  
echo "   • Statistical confidence intervals"
echo "   • Comparison across benchmark runs"