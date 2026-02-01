#!/usr/bin/env python3
"""
Test runner for RemDB Python bindings.

This script discovers and runs all tests in the tests/ directory.
It supports both unit tests and integration tests.

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
"""

import argparse
import os
import sys
import unittest
from pathlib import Path

# Add the current directory to Python path so tests can import remdb
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


def run_tests(args):
    """Run tests based on command line arguments."""
    
    # Determine start directory
    start_dir = args.start_directory
    if not os.path.exists(start_dir):
        print(f"Error: Start directory '{start_dir}' does not exist.")
        return 1
    
    # Handle test type filtering
    if args.integration:
        start_dir = os.path.join(start_dir, 'integration')
    elif args.unit:
        start_dir = os.path.join(start_dir, 'unit')
    
    if not os.path.exists(start_dir):
        print(f"Warning: Directory '{start_dir}' does not exist. No tests to run.")
        return 0
    
    # Setup test loader
    loader = unittest.TestLoader()
    
    # Apply pattern
    pattern = args.pattern
    
    # Apply exclude patterns
    if args.exclude:
        for exclude_pattern in args.exclude.split(','):
            loader.testNamePatterns = [f'*{exclude_pattern}*']
    
    # Discover tests
    print(f"Discovering tests in {start_dir} with pattern '{pattern}'...")
    suite = loader.discover(start_dir, pattern=pattern)
    
    if args.list:
        # List tests without running
        print("\nDiscovered tests:")
        for test in iterate_tests(suite):
            print(f"  {test}")
        print(f"\nTotal tests: {suite.countTestCases()}")
        return 0
    
    # Setup test runner
    if args.verbose:
        verbosity = 2
    elif args.quiet:
        verbosity = 0
    else:
        verbosity = 1
    
    runner_args = {
        'verbosity': verbosity,
        'failfast': args.failfast,
        'buffer': args.buffer,
        'tb_locals': args.locals,
    }
    
    # Run with coverage if requested
    if args.coverage or args.coverage_html or args.coverage_xml:
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
            
            # Generate reports
            if args.coverage_report or args.coverage_html or args.coverage_xml:
                print("\n" + "="*60)
                print("Coverage Report")
                print("="*60)
                cov.report(show_missing=True)
                
                if args.coverage_html:
                    html_dir = 'htmlcov'
                    print(f"\nGenerating HTML report to {html_dir}/...")
                    cov.html_report(directory=html_dir)
                    
                if args.coverage_xml:
                    xml_file = 'coverage.xml'
                    print(f"\nGenerating XML report to {xml_file}...")
                    cov.xml_report(outfile=xml_file)
            
            return 0 if result.wasSuccessful() else 1
            
        except ImportError:
            print("Warning: coverage module not installed. Running tests without coverage.")
            print("Install with: pip install coverage")
            runner = unittest.TextTestRunner(**runner_args)
            result = runner.run(suite)
            return 0 if result.wasSuccessful() else 1
    else:
        # Run without coverage
        runner = unittest.TextTestRunner(**runner_args)
        result = runner.run(suite)
        return 0 if result.wasSuccessful() else 1


def iterate_tests(test_suite):
    """Iterate over all tests in a test suite."""
    for test in test_suite:
        if isinstance(test, unittest.TestSuite):
            yield from iterate_tests(test)
        else:
            yield test.id()


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
    
    args = parser.parse_args()
    
    # Validate arguments
    if args.coverage_report and not (args.coverage or args.coverage_html or args.coverage_xml):
        parser.error("--coverage-report requires --coverage, --coverage-html, or --coverage-xml")
    
    if args.coverage_html or args.coverage_xml:
        args.coverage = True
    
    if args.quiet and args.verbose:
        parser.error("Cannot specify both --quiet and --verbose")
    
    if args.integration and args.unit:
        parser.error("Cannot specify both --integration and --unit")
    
    # Run tests
    return run_tests(args)


if __name__ == '__main__':
    sys.exit(main())