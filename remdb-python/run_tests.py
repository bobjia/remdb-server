#!/usr/bin/env python3
"""
Test runner for RemDB Python bindings.

This script discovers and runs all tests in the tests/ directory.
It supports both unittest and pytest frameworks.

Usage:
    python run_tests.py [options]

Options:
    -h, --help          Show this help message
    -v, --verbose       Verbose output
    -q, --quiet         Quiet output (only show failures)
    --pattern PATTERN   Pattern to match test files (default: test*.py)
    --start-directory   Directory to start discovery (default: tests)
    --coverage          Run with coverage reporting
    --coverage-html     Generate HTML coverage report
    --coverage-xml      Generate XML coverage report
    --coverage-report   Show coverage report (requires --coverage)
    --list              List discovered tests without running them
    --failfast          Stop on first failure
    --buffer            Buffer stdout/stderr during test runs
    --locals            Show local variables in tracebacks
    --exclude           Exclude patterns (comma separated)
    --integration       Run integration tests only
    --unit              Run unit tests only
    --pytest            Use pytest instead of unittest
    --parallel N        Run tests in parallel with N workers (pytest only)
    --marker MARKER     Run tests with specific pytest marker
    --timeout SECONDS   Set test timeout (pytest only)
    --report            Generate test report
    --report-html       Generate HTML test report
"""

import argparse
import os
import sys
import time
import traceback
from pathlib import Path
from typing import List, Optional, Dict, Any

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


class TestResult:
    """Test result container."""
    
    def __init__(self):
        self.total = 0
        self.passed = 0
        self.failed = 0
        self.errors = 0
        self.skipped = 0
        self.duration = 0.0
        self.failures: List[Dict[str, Any]] = []
        self.error_list: List[Dict[str, Any]] = []


class TestRunner:
    """Base test runner class."""
    
    def __init__(self, args):
        self.args = args
        self.result = TestResult()
    
    def run(self) -> int:
        raise NotImplementedError
    
    def print_summary(self):
        print("\n" + "=" * 60)
        print("Test Summary")
        print("=" * 60)
        print(f"Total tests:  {self.result.total}")
        print(f"Passed:       {self.result.passed}")
        print(f"Failed:       {self.result.failed}")
        print(f"Errors:       {self.result.errors}")
        print(f"Skipped:      {self.result.skipped}")
        print(f"Duration:     {self.result.duration:.2f}s")
        print("=" * 60)
        
        if self.result.failures:
            print("\nFailed tests:")
            for f in self.result.failures:
                print(f"  - {f['name']}: {f.get('message', 'No message')}")
        
        if self.result.error_list:
            print("\nErrors:")
            for e in self.result.error_list:
                print(f"  - {e['name']}: {e.get('message', 'No message')}")


class UnittestRunner(TestRunner):
    """Unittest test runner."""
    
    def run(self) -> int:
        import unittest
        
        start_dir = self._get_start_dir()
        if not start_dir:
            return 0
        
        loader = unittest.TestLoader()
        pattern = self.args.pattern
        
        if self.args.exclude:
            exclude_patterns = [f'*{p.strip()}*' for p in self.args.exclude.split(',')]
            loader.testNamePatterns = exclude_patterns
        
        print(f"Discovering tests in {start_dir} with pattern '{pattern}'...")
        suite = loader.discover(start_dir, pattern=pattern)
        
        if self.args.list:
            return self._list_tests(suite)
        
        verbosity = 2 if self.args.verbose else (0 if self.args.quiet else 1)
        
        runner_args = {
            'verbosity': verbosity,
            'failfast': self.args.failfast,
            'buffer': self.args.buffer,
            'tb_locals': self.args.locals,
        }
        
        start_time = time.time()
        
        if self.args.coverage or self.args.coverage_html or self.args.coverage_xml:
            result = self._run_with_coverage(suite, runner_args)
        else:
            runner = unittest.TextTestRunner(**runner_args)
            result = runner.run(suite)
        
        self.result.duration = time.time() - start_time
        self._collect_results(result)
        
        if self.args.report or self.args.report_html:
            self._generate_report()
        
        self.print_summary()
        
        return 0 if result.wasSuccessful() else 1
    
    def _get_start_dir(self) -> Optional[str]:
        start_dir = self.args.start_directory
        if not os.path.exists(start_dir):
            print(f"Error: Start directory '{start_dir}' does not exist.")
            return None
        
        if self.args.integration:
            start_dir = os.path.join(start_dir, 'integration')
        elif self.args.unit:
            start_dir = os.path.join(start_dir, 'unit')
        
        if not os.path.exists(start_dir):
            print(f"Warning: Directory '{start_dir}' does not exist. No tests to run.")
            return None
        
        return start_dir
    
    def _list_tests(self, suite) -> int:
        print("\nDiscovered tests:")
        for test in self._iterate_tests(suite):
            print(f"  {test}")
        print(f"\nTotal tests: {suite.countTestCases()}")
        return 0
    
    def _iterate_tests(self, test_suite):
        import unittest
        for test in test_suite:
            if isinstance(test, unittest.TestSuite):
                yield from self._iterate_tests(test)
            else:
                yield test.id()
    
    def _run_with_coverage(self, suite, runner_args):
        import unittest
        try:
            import coverage
            cov = coverage.Coverage(
                source=['remdb'],
                omit=['*/tests/*', '*/test*', '*/examples/*'],
                config_file=True
            )
            cov.start()
            
            runner = unittest.TextTestRunner(**runner_args)
            result = runner.run(suite)
            
            cov.stop()
            cov.save()
            
            if self.args.coverage_report or self.args.coverage_html or self.args.coverage_xml:
                print("\n" + "=" * 60)
                print("Coverage Report")
                print("=" * 60)
                cov.report(show_missing=True)
                
                if self.args.coverage_html:
                    html_dir = 'htmlcov'
                    print(f"\nGenerating HTML report to {html_dir}/...")
                    cov.html_report(directory=html_dir)
                
                if self.args.coverage_xml:
                    xml_file = 'coverage.xml'
                    print(f"\nGenerating XML report to {xml_file}...")
                    cov.xml_report(outfile=xml_file)
            
            return result
            
        except ImportError:
            print("Warning: coverage module not installed. Running tests without coverage.")
            print("Install with: pip install coverage")
            runner = unittest.TextTestRunner(**runner_args)
            return runner.run(suite)
    
    def _collect_results(self, result):
        self.result.total = result.testsRun
        self.result.passed = result.testsRun - len(result.failures) - len(result.errors) - len(result.skipped)
        self.result.failed = len(result.failures)
        self.result.errors = len(result.errors)
        self.result.skipped = len(result.skipped)
        
        for test, traceback_str in result.failures:
            self.result.failures.append({
                'name': str(test),
                'message': traceback_str.split('\n')[-2] if traceback_str else 'No message'
            })
        
        for test, traceback_str in result.errors:
            self.result.error_list.append({
                'name': str(test),
                'message': traceback_str.split('\n')[-2] if traceback_str else 'No message'
            })
    
    def _generate_report(self):
        report_dir = 'test_reports'
        os.makedirs(report_dir, exist_ok=True)
        
        if self.args.report_html:
            html_file = os.path.join(report_dir, 'test_report.html')
            self._generate_html_report(html_file)
            print(f"\nGenerated HTML report: {html_file}")
    
    def _generate_html_report(self, filepath):
        html = f"""<!DOCTYPE html>
<html>
<head>
    <title>RemDB Python Test Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        .summary {{ background: #f5f5f5; padding: 15px; border-radius: 5px; }}
        .passed {{ color: green; }}
        .failed {{ color: red; }}
        .errors {{ color: orange; }}
        .skipped {{ color: gray; }}
        table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #4CAF50; color: white; }}
        tr:nth-child(even) {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <h1>RemDB Python Test Report</h1>
    <div class="summary">
        <p><strong>Total tests:</strong> {self.result.total}</p>
        <p class="passed"><strong>Passed:</strong> {self.result.passed}</p>
        <p class="failed"><strong>Failed:</strong> {self.result.failed}</p>
        <p class="errors"><strong>Errors:</strong> {self.result.errors}</p>
        <p class="skipped"><strong>Skipped:</strong> {self.result.skipped}</p>
        <p><strong>Duration:</strong> {self.result.duration:.2f}s</p>
    </div>
"""
        
        if self.result.failures:
            html += """
    <h2>Failed Tests</h2>
    <table>
        <tr><th>Test</th><th>Message</th></tr>
"""
            for f in self.result.failures:
                html += f"        <tr><td>{f['name']}</td><td>{f.get('message', '')}</td></tr>\n"
            html += "    </table>\n"
        
        if self.result.error_list:
            html += """
    <h2>Errors</h2>
    <table>
        <tr><th>Test</th><th>Message</th></tr>
"""
            for e in self.result.error_list:
                html += f"        <tr><td>{e['name']}</td><td>{e.get('message', '')}</td></tr>\n"
            html += "    </table>\n"
        
        html += """
</body>
</html>
"""
        
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(html)


class PytestRunner(TestRunner):
    """Pytest test runner."""
    
    def run(self) -> int:
        try:
            import pytest
            from pytest import ExitCode
        except ImportError:
            print("Error: pytest is not installed.")
            print("Install with: pip install pytest")
            return 1
        
        start_dir = self._get_start_dir()
        if not start_dir:
            return 0
        
        pytest_args = self._build_pytest_args(start_dir)
        
        if self.args.list:
            pytest_args.insert(0, '--collect-only')
        
        start_time = time.time()
        exit_code = pytest.main(pytest_args)
        self.result.duration = time.time() - start_time
        
        if self.args.report or self.args.report_html:
            self._generate_report()
        
        self.print_summary()
        
        return 0 if exit_code == ExitCode.OK else 1
    
    def _get_start_dir(self) -> Optional[str]:
        start_dir = self.args.start_directory
        if not os.path.exists(start_dir):
            print(f"Error: Start directory '{start_dir}' does not exist.")
            return None
        
        if self.args.integration:
            start_dir = os.path.join(start_dir, 'integration')
        elif self.args.unit:
            start_dir = os.path.join(start_dir, 'unit')
        
        if not os.path.exists(start_dir):
            print(f"Warning: Directory '{start_dir}' does not exist. No tests to run.")
            return None
        
        return start_dir
    
    def _build_pytest_args(self, start_dir: str) -> List[str]:
        args = [start_dir]
        
        if self.args.verbose:
            args.append('-v')
        elif self.args.quiet:
            args.append('-q')
        
        if self.args.failfast:
            args.append('-x')
        
        if self.args.pattern:
            args.extend(['-k', self.args.pattern.replace('test*.py', '').replace('*', '')])
        
        if self.args.exclude:
            args.extend(['--ignore-glob', self.args.exclude])
        
        if self.args.parallel:
            try:
                import pytest_xdist
                args.extend(['-n', str(self.args.parallel)])
            except ImportError:
                print("Warning: pytest-xdist not installed. Running tests sequentially.")
        
        if self.args.marker:
            args.extend(['-m', self.args.marker])
        
        if self.args.timeout:
            args.extend(['--timeout', str(self.args.timeout)])
        
        if self.args.coverage or self.args.coverage_html or self.args.coverage_xml:
            try:
                import pytest_cov
                args.append('--cov=remdb')
                args.append('--cov-report=term-missing')
                
                if self.args.coverage_html:
                    args.append('--cov-report=html:htmlcov')
                
                if self.args.coverage_xml:
                    args.append('--cov-report=xml:coverage.xml')
            except ImportError:
                print("Warning: pytest-cov not installed. Running tests without coverage.")
        
        if self.args.report_html:
            try:
                import pytest_html
                args.extend(['--html', 'test_reports/pytest_report.html', '--self-contained-html'])
            except ImportError:
                print("Warning: pytest-html not installed. Skipping HTML report generation.")
        
        return args
    
    def _generate_report(self):
        report_dir = 'test_reports'
        os.makedirs(report_dir, exist_ok=True)
        print(f"\nTest reports generated in: {report_dir}/")


def main():
    parser = argparse.ArgumentParser(
        description="Run RemDB Python tests",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__
    )
    
    parser.add_argument('-v', '--verbose', action='store_true',
                        help='Verbose output')
    parser.add_argument('-q', '--quiet', action='store_true',
                        help='Quiet output (only show failures)')
    parser.add_argument('--pattern', default='test*.py',
                        help='Pattern to match test files (default: test*.py)')
    parser.add_argument('--start-directory', default='tests',
                        help='Directory to start discovery (default: tests)')
    parser.add_argument('--coverage', action='store_true',
                        help='Run with coverage reporting')
    parser.add_argument('--coverage-html', action='store_true',
                        help='Generate HTML coverage report')
    parser.add_argument('--coverage-xml', action='store_true',
                        help='Generate XML coverage report')
    parser.add_argument('--coverage-report', action='store_true',
                        help='Show coverage report (requires --coverage)')
    parser.add_argument('--list', action='store_true',
                        help='List discovered tests without running them')
    parser.add_argument('--failfast', action='store_true',
                        help='Stop on first failure')
    parser.add_argument('--buffer', action='store_true',
                        help='Buffer stdout/stderr during test runs')
    parser.add_argument('--locals', action='store_true',
                        help='Show local variables in tracebacks')
    parser.add_argument('--exclude',
                        help='Exclude patterns (comma separated)')
    parser.add_argument('--integration', action='store_true',
                        help='Run integration tests only')
    parser.add_argument('--unit', action='store_true',
                        help='Run unit tests only')
    parser.add_argument('--pytest', action='store_true',
                        help='Use pytest instead of unittest')
    parser.add_argument('--parallel', type=int,
                        help='Run tests in parallel with N workers (pytest only)')
    parser.add_argument('--marker',
                        help='Run tests with specific pytest marker')
    parser.add_argument('--timeout', type=int,
                        help='Set test timeout in seconds (pytest only)')
    parser.add_argument('--report', action='store_true',
                        help='Generate test report')
    parser.add_argument('--report-html', action='store_true',
                        help='Generate HTML test report')
    
    args = parser.parse_args()
    
    if args.coverage_report and not (args.coverage or args.coverage_html or args.coverage_xml):
        parser.error("--coverage-report requires --coverage, --coverage-html, or --coverage-xml")
    
    if args.coverage_html or args.coverage_xml:
        args.coverage = True
    
    if args.quiet and args.verbose:
        parser.error("Cannot specify both --quiet and --verbose")
    
    if args.integration and args.unit:
        parser.error("Cannot specify both --integration and --unit")
    
    if args.parallel and not args.pytest:
        print("Warning: --parallel only works with --pytest. Using unittest instead.")
    
    if args.pytest:
        runner = PytestRunner(args)
    else:
        runner = UnittestRunner(args)
    
    return runner.run()


if __name__ == '__main__':
    sys.exit(main())
