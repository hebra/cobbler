<dev-standards>
  <instructions>
    - Provide clean, efficient and simple code solutions.
    - Avoid over-engineering.
    - Prioritise consistency across existing code.
    - Follow code conventions and best practices for the specific technology stack being used.
    - Consider edge cases and potential issues in your implementations.
    - Add unit tests to ensure code reliability.
    - Self-explanatory code is preferred with no or limited comments.
    - Only add inline comments in the code if the code is too complex to understand without it.
    Explanations via inline comments should be extremely scarce in the codebase.
    - Structure your code in a way that is easy to write unit tests for.
    - Avoid type, const or enum duplication.
  </instructions>

<output_format>
<code_blocks>
Use markdown code blocks with appropriate language syntax highlighting.
Modularise where possible.
Prefer functional coding over inheritance in most cases. However, when state or context within
a single entity is required, try to use classes.
If state is required to be kept across the entire system in the currently worked on project
segment, refer to the best practice state management solution for the used tech stack.
</code_blocks>
</output_format>

<error_handling>
- Code defensively, making sure that possible pitfalls are covered around incorrect inputs that
could make the code crash. However, do not over engineer the solution if the user instructs you
that error handling will be covered at a later stage. In that case, as part of your task
handover documentation add a list of recommendations of checks that should be added to avoid
likely code crashes.
</error_handling>

<unit_testing>
<unit_testing_rules>
When writing any unit tests in any tech stack:
- Each unit test's purpose is to test that the specific unit's code behaves a certain way,
covering majority of outliers and edge cases.
- Unit tests are cheaper and faster to run than integration or end-to-end tests. Hence, focus
on writing the majority of scenarios to test in the unit tests.
- Stick to the existing testing style already used in the project for that tech stack. For
reference, look at 2 testing files of your choice, or the ones advised to you as part of the
cline rules for that tech stack.
- Mock external functions or tools using existing mocking style. Keep mocking minimal. Only
specify mock implementations for what is actually required.
- Do not add comments to the unit tests. They should be self explanatory based on well chosen
variable names.
- Group tests for each function or logical user experience so that they are easier to read.
- Keep tests small and short. Write more smaller tests rather than a few long complex
inter-dependant tests.
- Make sure every unit test can run independently of other unit tests running. Unit tests
should not have an order and should not disrupt each other's state if the order of execution
of the tests changes.
</unit_testing_rules>

    <unit_testing_instructions>
      1. Determine which file you are testing and understand the file and its functional purpose
      thoroughly.
      2. Devise a structural unit testing plan that includes tests for the majority of lines of
      code, code blocks and logical statements in the file.
      3. Clarify any uncertainties with the user before writing the tests and revise your plan
      accordingly until your confidence in your plan correctly adhering to the notes given above
      is at least 8 out of 10.
      4. Write the first unit test.
      5. Run each test after writing it to make sure it passes.
      6. Once the test passes, change an input to make the test fail, to check that the test isn't
      always truthy. Revert it back to make the test pass again.
      7. Write the next unit test.
      8. Once all tests are written according to your plan, run them all to make sure that they
      don't fail when ran together.
      9. Make sure that coverage of the entire unit tests in the project passes.
    </unit_testing_instructions>
</unit_testing>

  <logging>
    - Use the existing logging solution in the project. If you do not know what the existing
    solution is or there is none, ask the user what they want you to do next.
    - Write basic logging for your code.
    - Include important variables that make it easier for the reader to understand the state of the
    code for the logged line.
    - Add debug logs for areas that are more likely to break or need to be checked if errors occur
    in the future.
    - When writing logs that are exposed in a browser, keep them minimal and make sure to not expose
    any information that can help hackers compromise the system.
    - Always add a log as part of error handling code.
    - Never log passwords, credentials, tokens, PII, or other secure information.
    - If you are unsure about how or where to log, do your best and focus on just minimal logging.
    Do not dwell on logging too much.
  </logging>
</dev-standards>

<GIT_SEMANTIC_MESSAGES>
<documentation>
This section contains mandatory rules for writing semantic commit messages following the Conventional Commits specification.
These rules ensure consistent, meaningful, and parseable commit messages across all projects.
</documentation>

<CRITICAL_REQUIREMENTS>
<RULE id="GSM001" priority="MANDATORY">All commit messages MUST follow the semantic format: type(scope): description</RULE>
<RULE id="GSM002" priority="MANDATORY">Commit type MUST be one of the approved types listed below</RULE>
<RULE id="GSM003" priority="MANDATORY">Description MUST be written in imperative mood (e.g., "add feature" not "added feature")</RULE>
<RULE id="GSM004" priority="MANDATORY">Description MUST NOT end with a period</RULE>
<RULE id="GSM005" priority="MANDATORY">Type and scope MUST be lowercase</RULE>
</CRITICAL_REQUIREMENTS>

<COMMIT_MESSAGE_FORMAT>
<STRUCTURE>
<LINE1>type(scope): description</LINE1>
<BLANK_LINE/>
<BODY>Optional body providing additional context</BODY>
<BLANK_LINE/>
<FOOTER>Optional footer for breaking changes, issue references</FOOTER>
</STRUCTURE>

    <CONSTRAINTS>
      <RULE>First line MUST be ≤ 50 characters</RULE>
      <RULE>Body lines SHOULD be ≤ 72 characters</RULE>
      <RULE>Scope is optional but recommended for clarity</RULE>
      <RULE>Use blank lines to separate header, body, and footer</RULE>
    </CONSTRAINTS>
</COMMIT_MESSAGE_FORMAT>

<APPROVED_COMMIT_TYPES>
<TYPE name="feat">
<DESCRIPTION>New feature for the user</DESCRIPTION>
<EXAMPLES>
- feat(auth): add OAuth2 login support
- feat: implement user dashboard
- feat(api): add user profile endpoints
</EXAMPLES>
</TYPE>

    <TYPE name="fix">
      <DESCRIPTION>Bug fix for the user</DESCRIPTION>
      <EXAMPLES>
        - fix(login): resolve authentication timeout issue
        - fix: correct calculation in payment processor
        - fix(ui): fix button alignment on mobile devices
      </EXAMPLES>
    </TYPE>
    
    <TYPE name="docs">
      <DESCRIPTION>Documentation changes</DESCRIPTION>
      <EXAMPLES>
        - docs: update API documentation
        - docs(readme): add installation instructions
        - docs: fix typos in contributing guide
      </EXAMPLES>
    </TYPE>
    
    <TYPE name="style">
      <DESCRIPTION>Code style changes (formatting, missing semicolons, whitespace)</DESCRIPTION>
      <EXAMPLES>
        - style: fix indentation in user service
        - style(css): organise stylesheet imports
        - style: remove trailing whitespace
      </EXAMPLES>
    </TYPE>
    
    <TYPE name="refactor">
      <DESCRIPTION>Code refactoring without changing functionality</DESCRIPTION>
      <EXAMPLES>
        - refactor(auth): extract validation logic to separate module
        - refactor: simplify user data processing
        - refactor(db): optimise query structure
      </EXAMPLES>
    </TYPE>
    
    <TYPE name="test">
      <DESCRIPTION>Adding or updating tests</DESCRIPTION>
      <EXAMPLES>
        - test(auth): add unit tests for login validation
        - test: increase coverage for payment module
        - test(e2e): add user registration flow tests
      </EXAMPLES>
    </TYPE>
    
    <TYPE name="chore">
      <DESCRIPTION>Maintenance tasks, dependency updates, build changes</DESCRIPTION>
      <EXAMPLES>
        - chore: update dependencies to latest versions
        - chore(build): configure webpack for production
        - chore: add pre-commit hooks
      </EXAMPLES>
    </TYPE>
    
    <TYPE name="perf">
      <DESCRIPTION>Performance improvements</DESCRIPTION>
      <EXAMPLES>
        - perf(db): optimise user query performance
        - perf: reduce bundle size by 15%
        - perf(api): implement response caching
      </EXAMPLES>
    </TYPE>
    
    <TYPE name="ci">
      <DESCRIPTION>CI/CD pipeline changes</DESCRIPTION>
      <EXAMPLES>
        - ci: add automated testing workflow
        - ci(deploy): configure staging environment
        - ci: update Node.js version in pipeline
      </EXAMPLES>
    </TYPE>
    
    <TYPE name="build">
      <DESCRIPTION>Build system changes</DESCRIPTION>
      <EXAMPLES>
        - build: update webpack configuration
        - build(docker): optimise container image size
        - build: add development environment setup
      </EXAMPLES>
    </TYPE>
    
    <TYPE name="revert">
      <DESCRIPTION>Reverting previous commits</DESCRIPTION>
      <EXAMPLES>
        - revert: "feat(auth): add OAuth2 login support"
        - revert(api): remove experimental endpoint
      </EXAMPLES>
    </TYPE>
</APPROVED_COMMIT_TYPES>

<SCOPE_GUIDELINES>
<RULE>Scope SHOULD indicate the area of codebase affected</RULE>
<RULE>Use consistent scope names across the project</RULE>
<RULE>Common scopes include: api, ui, auth, db, config, docs, test</RULE>
<RULE>Scope MUST be lowercase and contain no spaces</RULE>

    <EXAMPLES>
      <GOOD>
        - feat(api): add user endpoints
        - fix(auth): resolve token validation
        - docs(readme): update setup instructions
      </GOOD>
      <BAD>
        - feat(API): add user endpoints  <!-- uppercase scope -->
        - fix(user auth): resolve token  <!-- space in scope -->
        - docs(Read Me): update setup    <!-- mixed case with space -->
      </BAD>
    </EXAMPLES>
</SCOPE_GUIDELINES>

<DESCRIPTION_RULES>
<RULE id="DESC001">Use imperative mood ("add" not "added" or "adds")</RULE>
<RULE id="DESC002">Start with lowercase letter</RULE>
<RULE id="DESC003">Do not end with period</RULE>
<RULE id="DESC004">Be concise but descriptive</RULE>
<RULE id="DESC005">Focus on what the change does, not how</RULE>

    <EXAMPLES>
      <GOOD>
        - add user authentication middleware
        - fix memory leak in data processor
        - update API documentation for v2
        - remove deprecated configuration options
      </GOOD>
      <BAD>
        - Added user authentication middleware.     <!-- past tense, period -->
        - Fixes memory leak in data processor      <!-- wrong tense -->
        - Updated API documentation for v2.        <!-- past tense, period -->
        - Removing deprecated configuration        <!-- wrong tense -->
      </BAD>
    </EXAMPLES>
</DESCRIPTION_RULES>

<BREAKING_CHANGES>
<RULE>Breaking changes MUST be indicated with "!" after type/scope</RULE>
<RULE>Breaking changes MUST be described in commit footer</RULE>
<RULE>Use "BREAKING CHANGE:" prefix in footer</RULE>

    <FORMAT>
      <HEADER>feat(api)!: remove deprecated user endpoints</HEADER>
      <BODY>
        Remove legacy user management endpoints that have been
        deprecated since v2.0. All clients should migrate to v3 API.
      </BODY>
      <FOOTER>
        BREAKING CHANGE: /api/v1/users endpoints removed.
        Use /api/v3/users instead.
      </FOOTER>
    </FORMAT>
</BREAKING_CHANGES>

<BODY_AND_FOOTER_GUIDELINES>
<BODY_RULES>
<RULE>Use body to explain what and why, not how</RULE>
<RULE>Wrap lines at 72 characters</RULE>
<RULE>Separate body from header with blank line</RULE>
<RULE>Use present tense and imperative mood</RULE>
</BODY_RULES>

    <FOOTER_RULES>
      <RULE>Use footer for issue references and breaking changes</RULE>
      <RULE>Format: "Fixes #123" or "Closes #456"</RULE>
      <RULE>Multiple issues: "Fixes #123, #456"</RULE>
      <RULE>Breaking changes: "BREAKING CHANGE: description"</RULE>
    </FOOTER_RULES>
</BODY_AND_FOOTER_GUIDELINES>

<COMPLETE_EXAMPLES>
<EXAMPLE type="simple">
<COMMIT>feat(auth): add two-factor authentication</COMMIT>
</EXAMPLE>

    <EXAMPLE type="with_body">
      <COMMIT>
        fix(api): resolve race condition in user creation

        The user creation endpoint was experiencing race conditions
        when multiple requests were made simultaneously. This fix
        implements proper locking to ensure data consistency.

        Fixes #234
      </COMMIT>
    </EXAMPLE>
    
    <EXAMPLE type="breaking_change">
      <COMMIT>
        feat(api)!: redesign authentication system

        Completely redesign the authentication system to use JWT
        tokens instead of session-based authentication. This provides
        better scalability and stateless operation.

        BREAKING CHANGE: Session-based authentication removed.
        All clients must migrate to JWT token authentication.
        See migration guide in docs/auth-migration.md

        Fixes #123, #456
      </COMMIT>
    </EXAMPLE>
</COMPLETE_EXAMPLES>

<ENFORCEMENT_RULES>
<RULE id="ENF001">ALL commits MUST follow semantic format before merging</RULE>
<RULE id="ENF002">Pull requests with non-semantic commits MUST be rejected</RULE>
<RULE id="ENF003">Use git hooks or CI checks to validate commit messages</RULE>
<RULE id="ENF004">Squash commits in pull requests to maintain clean history</RULE>
<RULE id="ENF005">Rebase and fix commit messages before merging if needed</RULE>
</ENFORCEMENT_RULES>

<COMMON_MISTAKES>
<MISTAKE>
<BAD>Fixed bug in login system</BAD>
<GOOD>fix(auth): resolve login validation error</GOOD>
<REASON>Missing type format, past tense, not specific enough</REASON>
</MISTAKE>

    <MISTAKE>
      <BAD>feat(API): Add new endpoints.</BAD>
      <GOOD>feat(api): add user management endpoints</GOOD>
      <REASON>Uppercase scope, period at end, not specific</REASON>
    </MISTAKE>
    
    <MISTAKE>
      <BAD>update documentation</BAD>
      <GOOD>docs: update API reference documentation</GOOD>
      <REASON>Missing type, not specific enough</REASON>
    </MISTAKE>
    
    <MISTAKE>
      <BAD>chore: Updated dependencies and fixed some bugs</BAD>
      <GOOD>chore: update project dependencies</GOOD>
      <REASON>Past tense, mixing multiple changes in one commit</REASON>
    </MISTAKE>
</COMMON_MISTAKES>

<INTEGRATION_WITH_TOOLS>
<CONVENTIONAL_CHANGELOG>
<RULE>Semantic commits enable automatic changelog generation</RULE>
<RULE>feat commits become "Features" section</RULE>
<RULE>fix commits become "Bug Fixes" section</RULE>
<RULE>Breaking changes get prominent placement</RULE>
</CONVENTIONAL_CHANGELOG>

    <SEMANTIC_VERSIONING>
      <RULE>feat commits trigger minor version bump</RULE>
      <RULE>fix commits trigger patch version bump</RULE>
      <RULE>Breaking changes trigger major version bump</RULE>
    </SEMANTIC_VERSIONING>
</INTEGRATION_WITH_TOOLS>

<VALIDATION_CHECKLIST>
<CHECKLIST>
<ITEM>✓ Commit message follows type(scope): description format</ITEM>
<ITEM>✓ Type is one of the approved types</ITEM>
<ITEM>✓ Scope is lowercase and relevant (if used)</ITEM>
<ITEM>✓ Description uses imperative mood</ITEM>
<ITEM>✓ Description starts with lowercase letter</ITEM>
<ITEM>✓ Description does not end with period</ITEM>
<ITEM>✓ First line is ≤ 50 characters</ITEM>
<ITEM>✓ Body lines are ≤ 72 characters (if used)</ITEM>
<ITEM>✓ Breaking changes are properly marked and documented</ITEM>
<ITEM>✓ Issue references are included in footer (if applicable)</ITEM>
</CHECKLIST>
</VALIDATION_CHECKLIST>
</GIT_SEMANTIC_MESSAGES>

<FILE_OPERATIONS>
<RULE id="FO001">Check existing project files before suggesting structural changes</RULE>
<RULE id="FO002">Complete multiple changes file efficiently - aim to perform all changes required in a single file within the same write</RULE>
<RULE id="FO003">Update documentation and fix errors in single edit per file</RULE>
</FILE_OPERATIONS>
