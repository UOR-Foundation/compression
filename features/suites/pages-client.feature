# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: pages-client

  @UC-WEB-01 @build
  Scenario: The Pages client is generated from and conforms to its Prism product model in Uorc.Page: stakeholders, features with their calls and acceptance cases, journeys, the page's UML state machine, operations, fields, controls, examples, error hints, sections, usage, links and quality measures. The generated page holds exactly the modeled elements, and a real browser walks every journey, takes every modeled transition, checks every control's enabled state in every state it reaches and holds every gated quality measure.
    Given the Uorc.Page module, the generated web/index.html and the site built from the release wasm
    When the model gate checks and renders the model and the browser acceptance drives the built site in headless Chromium
    Then every Prism check holds, the committed page equals the rendered model, every acceptance case passes, every transition is taken and every gated measure is within its threshold
