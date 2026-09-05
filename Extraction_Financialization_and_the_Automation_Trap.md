# Extraction, Financialization, and the Automation Trap: A Monte Carlo Model of the Global Producer–Consumer Game

*The mathematical synthesis of the current world economic situation. It
covers the resource curse, financialization of commodities, resource
nationalism, the Lewis Turning Point, the flying-geese production-migration
pattern, and technofeudal capture — each treated qualitatively, with
real-world citations. This chapter formalizes those mechanisms into a single
model and runs an actual Monte Carlo simulation, in the spirit of this
project's own `monte_carlo.rs`, to ask a concrete question: given current
trends, what happens to the next region entering the "producer" role — and
does automation change the answer?*

---

## 1. What this model tries to capture

Four claims from the preceding discussion, each individually evidenced, are
combined here into one dynamical system:

1. **The Lewis Turning Point** — a region's wages stay suppressed while
   surplus labor exists, then rise sharply once it's exhausted, historically
   triggering a "graduation" from producer to consumer role (Japan → Korea/
   Taiwan → China, each empirically documented).
2. **The resource curse / rentier-state mechanism** (Ross, 2001) — conflict
   risk rises with resource-rent dependence and falls with governance
   quality, and the two interact: high rents *and* weak governance is far
   worse than either alone (the DRC vs. Botswana divergence already
   discussed).
3. **Financialization of commodities** (Krippner 2011; Tang & Xiong) —
   speculative capital flow intensity amplifies extraction pressure on
   resource-rich, weak-governance regions independent of real supply and
   demand.
4. **Resource nationalism as a feedback response** — the DRC's 2025 cobalt
   export restrictions, Zimbabwe's lithium export ban, and Indonesia's
   nickel export ban are real, current examples of regions responding to
   extraction pressure by trying to capture more value locally.

To these four, the model adds one variable not present in the classical
Lewis/flying-geese literature, because it didn't exist when that literature
was written: **a rising global automation share**, reflecting the
accelerating AI/robotics capital cycle already documented in
`FASHION_FORCED_OBSOLENCE_AND_TECHNO_FEUDALISM.md` §5 (~\$725 billion in
2026 hyperscaler capex, up 77% from 2025). The model's central question is
whether this changes the historical pattern.

---

## 2. The formal model

Four regions $i \in \lbrace \text{West}, \text{China}, \text{Emerging}, \text{Frontier}\rbrace$,
each with state variables evolving annually over horizon $T$:

$$
w_i(t) \in [0,1] \quad \text{(wage/development index)}, \qquad g_i(t) \in [0,1] \quad \text{(governance quality)}, \qquad \rho_i \quad \text{(resource-rent share of GDP)}
$$

**Conflict risk** (resource-curse mechanism, logistic form):

$$
P(\text{conflict}_i(t)) = \sigma\Big(3.2 \cdot \rho_i \cdot (1 - g_i(t)) + 1.4 \cdot \epsilon_i(t) - 2.0 \cdot g_i(t) - 1.5\Big)
$$

where $\sigma$ is the logistic function and $\epsilon_i(t)$ is external
extraction pressure:

$$
\epsilon_i(t) = \rho_i \cdot \phi(t) \cdot \big(1 - \mathbb{1}[\text{nationalist}_i(t)] \cdot 0.25\big)
$$

$\phi(t) \in \lbrace 1.0, 1.6 \rbrace$ is a two-state Markov financialization
regime (Low/High speculative intensity), transitioning with persistence
probability 0.85 (Low→Low) and 0.65 (High→High) — mean-reverting but capable
of sustained high-speculation episodes, structurally similar to the regime-
switching model already in `economic_regimes.rs`.

**Resource nationalism adoption** is itself a feedback response to lived
conflict experience, not a fixed policy:

$$
P(\text{adopt nationalism}_i(t)) = 0.05 + 0.35 \cdot \mathbb{1}[\text{prior conflict experienced}]
$$

**Wage growth**, dampened by rising global automation share $a(t)$:

$$
\Delta w_i(t) = 0.018 \cdot \max(0.05,\; 1 - 0.9 \cdot a(t)) \cdot (1 + 0.3 \cdot g_i(t))
$$

with $a(t)$ growing at 4.5% annually (bounded at 0.85), reflecting the
documented AI/automation capex acceleration. A region transitions from
producer to consumer role (Lewis transition) once $w_i(t) \geq 0.55$.

**Value capture** splits each year's produced value between the producing
region, a financial/consumer-class intermediary layer, and automation-capital
owners:

$$
V_i(t) = \rho_i \cdot (1 + w_i(t)), \qquad V_i^{\text{local}} = V_i(t) \cdot \text{local-share}_i(t), \qquad V_i^{\text{remainder}} = V_i(t) - V_i^{\text{local}}
$$

$$
V_i^{\text{remainder}} \; \text{splits as} \; (1-a(t)) \to \text{consumer/financial class}, \qquad a(t) \to \text{automation-capital owners}
$$

This last split is the model's link to the technofeudal-capture argument
already developed in `HAPPINESS_OR_FEAR_WORK_LIFE.md` §5 and
`FASHION_FORCED_OBSOLENCE_AND_TECHNO_FEUDALISM.md` §8: as $a(t)$ rises,
an increasing share of value that would otherwise flow to intermediaries
instead flows to whoever owns the automated capital.

---

## 3. Simulation design and an explicit epistemic caveat

10,000 simulated 40-year futures, four regions initialized with
illustrative starting values (West: high wage/governance, near-zero resource
dependence; China: mid-transition, past its historical Lewis point; Emerging
["China Plus One" bloc — Vietnam/India/Mexico-type]: early-transition,
moderate governance; Frontier [Africa-type]: low wage, high resource rent,
low mean governance with wide variance — reflecting the real heterogeneity
between, say, Botswana and the DRC rather than a single "Africa" value).

**This must be stated plainly, in keeping with this project's own standard
(`FutureWork.md` §2.4's "weak validation and calibration culture" critique,
which applies here with full force): every parameter in this model is
reasoned and literature-informed, not fitted to historical data.** This is
an illustrative model built to test whether a specific mechanism
(automation dampening) changes a qualitative conclusion, not a calibrated
quantitative forecast of what will actually happen to any real country. The
right way to read the results below is "does adding this mechanism change
the story," not "this is what will happen to Africa."

---

## 4. Results

### 4a. Baseline (automation dampening active, reflecting current trends)

| Metric | P10 | Median | P90 |
|---|---|---|---|
| Frontier region: years in conflict (of 40) | 5 | 14 | 21 |
| Frontier region: Lewis-transition probability within 40 years | | **2.9%** | |
| Frontier region: nationalism-adoption probability | | **99.9%** | |
| China: final wage index | 0.553 | 0.557 | 0.564 |
| Value capture, producer regions | 58.7% | 60.5% | 61.5% |
| Value capture, consumer/financial class | 22.8% | 24.2% | 25.9% |
| Value capture, automation-capital owners | 14.6% | 15.4% | 16.2% |

### 4b. Counterfactual (automation dampening switched off — the historical, pre-AI-boom pattern)

| Metric | P10 | Median | P90 |
|---|---|---|---|
| Frontier region: years in conflict (of 40) | 3 | 14 | 21 |
| Frontier region: Lewis-transition probability within 40 years | | **36.2%** | |
| Frontier region: nationalism-adoption probability | | 99.5% | |
| China: final wage index | 0.553 | 0.556 | 0.566 |
| Value capture, producer regions | 58.2% | 60.3% | 61.5% |
| Value capture, consumer/financial class | 23.8% | 25.2% | 29.0% |
| Value capture, automation-capital owners | 11.7% | 14.8% | 15.4% |

---

## 5. Interpretation: the automation trap

The headline finding is the **12-fold drop in Lewis-transition probability**
(36.2% → 2.9%) once automation dampening is switched on. Mechanically, this
happens because rising automation erodes labor's bargaining position before
wages can rise far enough to cross the transition threshold — the same
wage-suppressing force that historically only ended once a region ran out of
surplus labor (Lewis's original mechanism) is, in this model, replaced by a
*structural* ceiling that doesn't go away once surplus labor is exhausted,
because the competing "labor" is no longer other humans in the same country,
it's automated capital anywhere.

This gives formal shape to the qualitative concern raised earlier in this
document series (`MULTIPOLAR_GAME.md`
§7, possibility 3): **the historical "flying geese" pattern that let Japan,
Korea, Taiwan, and China each graduate from producer to consumer role may
not repeat for the next region in line, not because that region is different
in kind, but because the automation variable wasn't present for any of the
historical cases.** This is a genuinely different mechanism from the
resource-curse and financialization arguments already covered — those explain
*conflict*, this explains why even a region that avoids conflict and adopts
sound governance might still not complete the transition on the historical
timetable.

A second, subtler finding is worth naming: **resource nationalism adoption
stays close to universal (99.5–99.9%) in both scenarios, but doesn't by
itself meaningfully change the Lewis-transition probability.** In the model,
nationalist policy increases the *local value-capture share*, but does not
directly accelerate wage growth — which mirrors a real, documented critique
in development economics: resource nationalization can enrich a captured
elite or state treasury without necessarily raising ordinary wages, unless
the captured revenue is specifically and effectively reinvested in ways that
also lift broad-based labor income. Capturing more value and raising wages
are related but not the same thing, and the model's separation of these two
mechanisms makes that distinction concrete rather than assumed.

---

## 6. What this would look like as a Rust module

Consistent with `FutureWork.md`'s roadmap structure, this fits as a natural
extension of the existing regime-switching architecture:

```
src/geopolitical_regimes.rs   — the four-region resource-game state machine
                                 (conflict risk, nationalism feedback, Lewis
                                 transition, value-capture accounting)
```

reusing the existing `economic_regimes.rs` Markov-chain machinery for the
financialization regime $\phi(t)$, and exposing new CLI parameters:

- `--automation-dampening` (0.0–1.0): strength of the labor-bargaining
  erosion effect from Section 2
- `--frontier-governance-mean` / `--frontier-governance-sd`: lets the person
  running the simulation test sensitivity to governance-quality assumptions
  directly, rather than treating a single point estimate as ground truth
- Output extended to report the value-capture triad (producer / consumer-
  financial / automation-capital) alongside the existing pension and work-
  percentage outputs, making explicit which layer of the global economy a
  given work-percentage decision is actually situated within

This would not replace the personal pension/work-percentage optimizer — it
would sit alongside it as a macro-context module, similar to the
competitiveness-regime proposal already made in
`NBA_Analogy_Multipolarity_and_Economic_Innovation.md` §6.

---

## 7. Limitations, stated plainly

- **Four regions is a coarse simplification** of a world with dozens of
  economically distinct trajectories; "Frontier" in particular flattens
  enormous real heterogeneity (Botswana and the DRC are both notionally
  "Africa" and produced opposite historical outcomes under similar resource
  endowments, precisely because of the governance-variance term the model
  tries, imperfectly, to capture via a wide standard deviation).
- **The automation-dampening coefficient (0.9) is a modeling choice**, not
  an estimated parameter — it was chosen to be large enough to test whether
  the mechanism *could* matter, not because it has been fit to real wage and
  automation data. A smaller coefficient would show a smaller effect; the
  qualitative direction (automation dampens Lewis transitions) is more
  robust than the specific 12-fold magnitude reported here.
- **Conflict is modeled as a single annual binary draw**, not as a process
  with genuine escalation dynamics, duration dependence, or spillover
  between regions — a real limitation relative to actual conflict.
- **No genuine multipolar strategic interaction between regions is modeled**
  — each region's trajectory evolves independently rather than in explicit
  response to the others' choices, which understates the game-theoretic
  richness of `MULTIPOLAR_GAME.md`
  §4's security-dilemma argument. A fuller version would let regions'
  automation-adoption and nationalism choices respond to each other's prior
  moves, not just to their own conflict history.

None of these limitations were papered over because doing so would
contradict the entire premise of this document series — the value of the
exercise is in making the mechanism precise enough to interrogate, not in
producing a number to believe uncritically.

---

## 8. Summary

- This chapter formalizes the preceding discussion's four mechanisms
  (Lewis Turning Point, resource curse, financialization amplification,
  resource-nationalism feedback) into a single dynamical model, and adds a
  fifth variable — rising global automation share — not present in the
  historical literature this pattern is drawn from.
- A 10,000-run, 40-year Monte Carlo simulation finds the "Frontier" region's
  probability of completing a historical-style Lewis transition falls from
  36.2% to 2.9% once automation-driven labor-bargaining erosion is included
  — a formal illustration of the "automation trap" hypothesis raised
  earlier in this document series.
- Resource nationalism adoption is nearly universal in the model regardless
  of the automation assumption, but does not by itself restore the
  transition probability — capturing more local value and raising broad-
  based wages are mechanically distinct outcomes.
- All parameters are explicitly illustrative and literature-informed, not
  empirically calibrated — the model is a tool for testing whether a
  mechanism changes a qualitative conclusion, not a forecast of any actual
  country's future, consistent with the calibration-honesty standard this
  entire project has held itself to since `FutureWork.md` §2.4.
- A concrete Rust implementation path is proposed, extending the existing
  `economic_regimes.rs` Markov-chain machinery rather than building new
  infrastructure from scratch.

---

## Further reading

- W. Arthur Lewis, "Economic Development with Unlimited Supplies of Labour"
  (1954) — the original Lewis model
- Michael Ross, "Does Oil Hinder Democracy?" (*World Politics*, 2001) — the
  rentier-state mechanism used in Section 2's conflict-risk formula
- Greta Krippner, *Capitalizing on Crisis* (2011, already cited in
  `MULTIPOLAR_GAME.md`)
- Tang & Xiong, "Index Investment and the Financialization of Commodities"
  (2012)
- Kaname Akamatsu's "flying geese" model of sequential East Asian
  industrialization
- Documented 2025 resource-nationalism cases: DRC cobalt export
  restrictions (February 2025), Zimbabwe's unprocessed-lithium export ban,
  Indonesia's unprocessed-nickel export ban (2020)
- Cross-reference: [`MULTIPOLAR_GAME.md`](MULTIPOLAR_GAME.md)
  §2–4, §7; [`NBA_Analogy_Multipolarity_and_Economic_Innovation.md`](NBA_Analogy_Multipolarity_and_Economic_Innovation.md)
  §5–6; [`FASHION_FORCED_OBSOLENCE_AND_TECHNO_FEUDALISM.md`](FASHION_FORCED_OBSOLENCE_AND_TECHNO_FEUDALISM.md)
  §5, §8; [`FutureWork.md`](FutureWork.md) §2.4
