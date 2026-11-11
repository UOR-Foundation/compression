# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: pages-client

  @UC-WEB-01 @build
  Scenario: The Pages client is generated from and conforms to its Prism product model in Uorc.Page: stakeholders, features with their calls and acceptance cases, journeys, the page's UML state machine, operations, fields, controls, examples, error hints, sections, usage, links and quality measures. The generated page holds exactly the modeled elements, and a real browser walks every journey, takes every modeled transition, checks every control's enabled state in every state it reaches and holds every gated quality measure.
    Given the Uorc.Page module, the generated web/index.html and the site built from the release wasm
    When the model gate checks and renders the model and the browser acceptance drives the built site in headless Chromium
    Then every Prism check holds, the committed page equals the rendered model, every acceptance case passes, every transition is taken and every gated measure is within its threshold

  @UC-WEB-02 @build
  Scenario: The product shows the codec as a point of its modelled space: uorc inspect, the library's Info JSON and the Pages client name each frame's codec point (basis, ncm profile word, consolidated model state, lanes and its κ) and the exact steps its decode takes, as Uorc.Performance defines them, from the header alone; the page's model view shows the release's performance theorems, its closed claims with codec point, corpus κ, bytes, steps and the factory's decision, and the engine's tick, rule30 register and weights, exactly as the release publishes them in model.json.
    Given an archive written by uorc, the Uorc.Page module and the site built with model.json from the claim ledger, frontier.toml and model/engine.toml
    When uorc inspect reads the archive and the browser acceptance inspects it on the page and reads the model view
    Then the codec point κ and steps equal the library's for every frame, and every theorem, claim and engine value on the page equals model.json
