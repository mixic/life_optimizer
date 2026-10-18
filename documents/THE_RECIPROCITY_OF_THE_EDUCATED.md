# The Reciprocity of the Educated

*A position, argued in the first person, on what a publicly funded qualification obliges its
holder to return — and on why the early-retirement and tax calculation is the place that
obligation either holds or quietly drains away.*

*Companion to `EARLY_RETIREMENT.md`, whose section 6 computes the ledgers this chapter
interprets and deliberately refuses to rank them. That refusal is right for a program. It is
not an answer for a person, and this chapter is the answer.*

---

## 1. Why this chapter exists

The early-retirement model contains a small machine called `ContributionPrinciple`. It prices
four different answers to the question *what should a person with a publicly funded education
contribute?*, prints the four ledgers side by side, and stops. The program is built that way on
purpose: it can compute what each principle implies, and it cannot decide which principle is
right without pretending that a value judgement is a derivation.

That is a defensible design for software and an insufficient one for a life. A person has to
choose. This chapter is my choice, stated as a choice, argued rather than assumed — and
located precisely, because the argument has a sharp edge that the rest of the repository makes
visible: **everything the model does to optimise an early retirement is also a machine for
reducing the measured discharge of this obligation.** That tension is not a flaw in the model.
It is the reason these two subjects belong in one chapter.

---

## 2. The claim

Three parts, and they are meant to be read together.

**(i) A publicly funded qualification is a transfer.** When a canton and the Confederation pay
for a university place, an apprenticeship, or a doctorate, they move resources from everyone —
including people who never received that qualification and whose taxes paid for it — to the
person who does. The size of the transfer varies enormously by qualification. That it is a
transfer does not.

**(ii) Receiving it creates an obligation of reciprocity.** Not a debt. Not gratitude in the
sense of a feeling. An obligation to return the benefit, because it was given.

**(iii) The obligation has two currencies: money and the use of the knowledge itself.** A
society that pays for education is not buying a service for the graduate. It is reproducing
its own capacity to think. The return it needs is therefore partly financial — so that the
institution can do it again for someone else — and partly *in kind*: the knowledge put to work
in ways the society that funded it can use.

I hold all three. The second is the load-bearing one, and section 4 explains why I ground the
obligation in reciprocity rather than in debt — because the two grounds have different
consequences, and the difference is not academic.

---

## 3. What I am *not* claiming

It matters what this position is not, because the sloppy versions are easier to attack than the
real one.

**This is not a claim about "intellectuals".** That word names a social category — people who
deal in ideas, or who are recognised as doing so — and it is contested, self-applied, and has
no administrative boundary. My claim is about something much narrower and fully checkable:
**the holder of a qualification that the public paid for.** A master's graduate and a certified
electrician are both inside it. A self-taught writer who never took public money is outside it,
however intellectual their work. Keeping the two apart is not pedantry; it is what makes the
claim a principle about public investment rather than a resentment about a class.

**This is not a claim that the educated are ungrateful.** The relevant facts are institutional:
whether the obligation is collected, and whether the instruments for collecting it are
defended. Gratitude is not collectable and not the point.

**This is not derived from the model's output.** The model's cost figures are *declared*, not
measured — there is no official per-qualification figure cited anywhere in it, and I say so in
`EARLY_RETIREMENT.md` §4.2 and again in section 9 below. The argument must therefore stand on
its structure. Any number the model produces is an illustration of the structure, never
evidence for it.

---

## 4. Why reciprocity, and not debt

This is the choice that decides the rest, so it has to be earned.

A **debt** is discharged by repayment and then it ends. Its size is the principal plus interest.
It is indifferent to what the debtor does afterwards, and it can be discharged in full by
someone who then does nothing useful with what they bought. If education were a debt, the
correct policy would be a graduate tax with an instalment schedule, a final payment, and a
certificate of discharge. It would also follow that a graduate who repaid and then spent forty
years doing harm would have discharged the obligation — which is absurd on the face of it,
because the harm is precisely what the public investment was supposed to prevent.

**Reciprocity** is a relation rather than a transaction. It is discharged by returning the
benefit *in kind* — by putting the knowledge to use where the society that funded it benefits —
and it does not terminate while the relation lasts. That is, in practice, until death.

Three consequences follow, and they are the reason I choose this ground:

**It explains why the money is not the whole obligation.** If a payment alone could settle it, a
wealthy graduate could buy their way out and contribute nothing else. I do not accept that, and
the reciprocity framing is what lets me refuse it.

**It explains why the money is not optional either.** An obligation discharged only in goodwill
fails on its own terms: the institution that produced the qualification needs resources to
produce the next one, and a generation of graduates who all contributed "in kind" and no money
would consume the system that made them. Reciprocity includes material return because the
benefit was material.

**It explains the shape of the obligation, which is progressive on two independent counts.**
Those who received more — a longer and costlier qualification — owe more. And those who are
positioned to return more, because the qualification raised their earnings, owe more. Both
scales point the same way. This is not a justification bolted on afterwards; it falls out of
the structure.

**It survives the reply I hear most often.** *"I paid my tuition."* Tuition is a **price**, and
in Switzerland it covers a small fraction of what a university place costs — the rest is the
public share. Paying a price does not repay a transfer; it is the condition of receiving one. A
reader who wants to reject this chapter should reject the premise that the public share is a
transfer, not the observation that fees are small relative to it.

---

## 5. Scope: any publicly funded qualification, scaled to what was paid

I want the rule to bind **everyone who received public money for a qualification**, scaled to
what they received — not only university graduates. This is a deliberate choice and it makes the
position stronger, not weaker, in three ways.

**It removes the arbitrariness.** A levy aimed only at university graduates invites the obvious
reply that it is a coalition taxing a rival. Switzerland funds its vocational path heavily
through the dual system; an apprentice's training is a public investment too. A principle that
says *public money creates reciprocity* cannot conveniently exempt the path taken by the
majority of the population, and if it did, the exemption would be evidence that the principle
was never the real reason.

**It turns a levy on a class into a principle about capital.** Stated generally, the claim is
that a society which invests in human capital is entitled to a return on it, from all of it.
That is a claim about public investment and can be argued as one. Stated narrowly, it is a claim
about a group, and it will be heard as envy whatever its author intended.

**It brings the vocational graduate inside the argument rather than outside it.** That matters
for the comparison in section 7, where the relevant benchmark turns out to be what a
*comparably paid* holder of a *less subsidised* qualification contributes — a comparison that
only becomes visible once the scope is general.

**The honest caveat.** The model's per-qualification costs are declared stand-ins, not measured
figures. Its ratios — that a doctorate is taken to cost the public several times a vocational
qualification — carry the premise, and every conclusion scales with them. So the *structure* of
this chapter's claim is robust to those numbers being wrong; its *size* is not. Anyone who
wants to attack the argument should attack it by replacing those numbers with real ones, which
is exactly what the model is built to accept.

---

## 6. What the calculation does with this

`src/early_retirement.rs` implements four principles and computes a ledger for each:

| Principle | What it says | What the ledger reports |
| --- | --- | --- |
| `AbilityToPay` | Contribute in proportion to income — what the existing progressive tax does | A surcharge of zero, and a multiple over the state's outlay |
| `EducationCostRecovery` | Recover the public outlay as a surcharge | The outlay spread over a declared recovery period |
| `BenefitReceived` | Contribute in proportion to the earnings premium | A declared share of lifetime gross income |
| `Flat` | Everyone contributes the same amount | A fixed annual sum, whose share of income **falls** as income rises |

In the default case the model ships with — a CHF 120,000 salary, a 35-year career, the Bern
schedule for a single person with no children, and a **declared** CHF 100,000 public cost for a
master's — the ordinary progressive schedule collects **CHF 613,932** of lifetime tax, which is
**6.1 times** the declared outlay.

That computation is the one hard fact this chapter has to work with, and its first important
property is what it does *not* do: it does not settle anything. A reader can look at 6.1 and
conclude the principle is honoured with room to spare, or conclude the measure is wrong in both
its numerator and its denominator. Section 7 is my argument for the second reading.

### 6.1 The tension that makes this chapter necessary

Here is the point I care most about, and it is the reason the early-retirement model and this
argument cannot be kept in separate documents.

**The early-retirement model is an optimiser whose objective is to pay less tax on the same
capital.** It allocates withdrawals across years to minimise the tax they attract, and its whole
apparatus — the linear program, the dual that gives the marginal rate of the next franc, the
water-filling solution that spreads a withdrawal until the next franc costs the same wherever it
goes — exists to do that well.

Measured by *lifetime tax*, as the ledger above measures it, that optimisation is also a machine
for minimising the discharge of the obligation this chapter argues for. Nothing in the model
distinguishes the two. It cannot, because to the model they are the same variable.

Three concrete links, each of which the model can quantify:

**Timing.** A capital benefit is taxed separately from ordinary income, as a full annual tax in
the year it is received (DBG Art. 38; StHG Art. 11 Abs. 3). Spreading the same capital across
more tax years therefore lowers the rate wherever the tariff is progressive. In the model's
default parameters the saving happens to be **nil**, because the whole withdrawal stays inside
the lowest band; but that is a property of the default capital, not of the method. The
condition under which timing starts to matter is precise and checkable: **the capital must be
large relative to the annual width of the cheap bands.** A larger pot, or a narrower first band,
and the saving appears. And in a canton with a flat capital tariff — Zurich and Thurgau are
effectively at 2% — timing buys nothing at all, so the opportunity is canton-dependent.

**Retirement age.** Every year of early retirement is a year of contributions not made, in both
currencies. The model shows the effect on tax directly. It cannot show the effect on the
in-kind half at all, because it does not measure in-kind contribution — which is the single
largest gap in the whole calculation (section 9).

**Annuitisation.** The share of capital taken as an annuity versus as a lump sum changes the
lifetime tax, and the CLI takes it as an argument (`--annuity-share`), so the tax column moves
with it. The risk table the CLI prints varies the *retirement age* and holds the annuity share
fixed, so the two effects are reported separately rather than confounded. Under this chapter's
principle, the choice is not only a risk decision. It is a decision about how much of the
obligation to discharge.

**What the model cannot say, and what I therefore must.** It can show the *size* of the
avoidance. It cannot say whether avoiding it is wrong. Tax-minimising timing is either
legitimate planning or under-discharge, and *the same facts support both readings*. That is not
a defect of the calculation. It is the exact point at which the calculation stops and the
judgement begins — and a person who lets the optimiser decide has, in effect, delegated a moral
choice to an objective function that was never told about it.

---

## 7. Why 6.1 times is not evidence of adequacy

I claim the current settlement **under-discharges** the obligation. I do not claim any particular
multiple is the right one — that is a public argument, not a derivation (section 8). What I claim
is narrower and, I think, harder to resist: **the figure of 6.1 is not evidence that the
obligation is met, because the measure is wrong in both directions, and both errors flatter the
graduate.**

**The numerator counts the wrong thing.** Six times the outlay is a multiple of *gross lifetime
tax*. But most of that tax funds things the graduate consumes himself: roads, defence, the
health system, and — eventually — his own pension. Only the fraction that reproduces the
education system is a return in the reciprocity sense. So gross tax massively overstates the
repayment. The correct numerator is smaller, and by a large factor.

**The denominator understates the cost.** The declared figure is the direct cost of the
qualification. It omits what the public also gave up: the graduate's foregone output during the
years of study, and the institutional and research capacity that a degree depends on and that
public money maintains. If those are added — as they should be, since they are part of what made
the qualification possible — the multiple falls immediately.

**Reciprocity does not terminate; tax does.** The obligation is a relation lasting a life. A
lifetime-tax measure stops counting the moment income stops, which is exactly when a retired
graduate's capacity to return in kind may be at its height. The measure cannot see the years it
most needs to see.

**The benchmark is wrong.** This is the argument that follows from the general scope in section
5. The multiple is computed against zero — against having received nothing. The right comparison
is against a **comparably paid holder of a less subsidised qualification**. If a certified
electrician earning the same income pays broadly the same tax while having consumed a much
smaller public subsidy, then the university graduate has received more and returned the same,
and his *net* position is better than a gross multiple of six suggests. Measuring against zero
hides precisely the comparison that the principle is about.

Taken together: the six is an artefact of a numerator that counts unrelated public services, a
denominator that omits real public costs, a clock that stops too early, and a benchmark set at
zero. Correcting any one of those reduces it. Correcting all four changes its meaning
completely. **"They already pay six times over" is therefore not an answer to this chapter. It
is a restatement of the accounting convention that produced it.**

That is the whole of my case for *more*. The size of the more is a separate question, and I
refuse to manufacture an answer to it.

---

## 8. What the position still owes

A position that has not met its strongest objections is not yet an argument. These are the ones
that bite, in the order they bite hardest.

**The ability-to-pay objection — the real one.** The settled principle of Swiss and most OECD
taxation is capacity to pay, not benefit received and not the cost of one's own formation. If
that principle is right, a great deal of this chapter is already satisfied by the existing
progressive tax, and the reform claim collapses. My answer is that the qualification is a
special case because it is the one large public investment whose return the recipient can
directly monetise *and* whose purpose is to create the very capacity that repays it. That is an
answer. Whether it is a principled boundary or a convenient one is the honest crux of the whole
position, and I do not think I can settle it by argument alone.

**The stopping-point objection.** It follows immediately: if the public cost of my education is
chargeable to me, why not the public cost of my schooling, my childhood health care, the roads
my parents used, or the maternity leave that let me exist? Once cost-recovery is admitted, what
stops it? My answer is the in-kind half — the obligation attaches to a qualification because a
qualification is a *transferable capacity*, not merely a benefit consumed — but I concede this
is the weakest joint in the position. A reader who presses here is pressing in the right place.

**The double-counting objection.** A progressive income tax already taxes the return to
education heavily. A surcharge on income taxes the same return twice. And here the position
turns on itself uncomfortably: the cleanest answer is that the reciprocity return should be
**in kind** rather than on income, since in-kind return is not double-counting at all. But that
is the answer that gives me least of what I want, because it makes the obligation largely
non-monetary in a chapter that also argues for more money. **I state this rather than resolve
it.** It is a real tension in my own position.

**The mobility objection.** A surcharge levied in one canton selects for the graduates who
leave, and the canton loses both the tax and the person. Swiss cantonal competition makes this
sharp, and any credible version of this policy needs a national base — or it must accept that
it is partly a subsidy to the jurisdictions that free-ride. An obligation that can be discharged
by moving is not much of an obligation.

**The family-background objection.** The strongest predictor of who obtains a master's is
parental background, not merit. A levy on graduates may therefore fall hardest on those whose
advantage was least, and leave untouched the inherited advantages that produced the outcome. My
answer is that the duty attaches to the *qualification*, which is publicly funded for everyone
who holds it regardless of how they came by it. That answers the *obligation*. It does not
answer the claim that the arrangement is *fair*, and I should not pretend otherwise.

**The care objection — and a limitation of the calculation.** Careers interrupted by caregiving
have lower lifetime income and therefore lower lifetime tax. A measure of contribution built on
lifetime tax penalises them for work that is itself a contribution — and work that the same
society depends on. The model computes contribution in money only and is structurally blind to
this. I regard this as the most serious defect in using the ledger as a measure of virtue, and I
have no fix for it within the current calculation.

**The cosmopolitan objection.** A graduate who emigrates may contribute more to humanity and less
to the state that paid. On my ground — reciprocity toward *the society that made it possible* —
emigration is under-discharge. A reader who holds that obligations run to humanity rather than to
nations will reject the premise, coherently, and there is no calculation that decides between us.

---

## 9. What the calculation can settle, and what it cannot

**It can compute:**

- the declared public cost of each qualification, and the ratio between them;
- lifetime tax under a given schedule, career and salary, and the multiple over that cost;
- the surcharge implied by each of the four principles, side by side;
- how much of the multiple is an artefact of the numerator, by recomputing against a narrower
  definition of return;
- the size of the timing advantage available from spreading a capital withdrawal in a given
  canton's tariff, and the condition under which it appears;
- the value of the contribution years that an early retirement removes.

**It cannot:**

- decide whether reciprocity grounds a duty at all, and against whom — the society, the state,
  or humanity;
- decide which of the two currencies is primary when they conflict, which is the
  double-counting question in another form;
- say what the right multiple is. A target multiple is a **declared political choice**; the model
  takes it as an input and reports what it costs, in the shape of
  `EducationCostRecovery { state_cost, recovery_years }`;
- see in-kind contribution at all. There is no parameter for hours taught, papers supervised,
  public service given, or code released. **This is the single largest gap between the
  calculation and the argument**, and it is the gap that most flatters the case for more money:
  a graduate who already returns a great deal in kind would appear in this model exactly like one
  who returns none;
- decide whether optimising the timing of a withdrawal is legitimate planning or
  under-discharge. It reports the size of the choice, not its moral status.

**And the standing warning.** Every cost figure in the model is declared, not measured. No number
in it is evidence about the world. The argument of this chapter stands on its structure — that a
public transfer creates a continuing reciprocal obligation, discharged in two currencies, scaled
to what was received — and that structure does not become stronger if the model's arithmetic
happens to be flattering. It would not become weaker if the arithmetic turned against it.

---

## 10. What could be built next, so the argument can be tested rather than asserted

I do not want this to remain a position that cannot be argued against. Three things would make it
falsifiable, and all three are small enough to implement:

**An in-kind ledger.** A declared number of hours a year given to teaching, supervision,
mentoring or public service, priced at the holder's own hourly rate as foregone income, and
reported beside the money. This closes the largest gap in section 9 and would let the claim
*"the current settlement under-discharges"* be tested rather than assumed. My honest expectation
is that it would weaken the money case for many graduates and strengthen it for a few — which is
a risk to my own conclusion that I am willing to run.

**A benchmark against the less-subsidised comparison.** Compute the same ledger for a
comparably paid holder of a much less subsidised qualification, and report the **difference**
rather than the level. That is the comparison section 7 argues is the right one, and the model
already has everything it needs except the second qualification's declared cost.

**A declared target multiple, and its cost.** Take a target as an input, and report what reaching
it would require from each principle. This does not make the target correct — nothing can — but
it makes the political argument arithmetic instead of rhetorical, which is the most a calculation
can honestly offer a moral claim.

---

## 11. The crux, stated plainly

I hold that a publicly funded qualification creates a continuing obligation of reciprocity; that
the obligation is discharged in money and in the use of the knowledge itself; that it binds
everyone who received public money, scaled to what they received, and not university graduates
alone; and that the present settlement under-discharges it.

Two of those claims carry the weight and are the least certain. That **reciprocity, not ability
to pay, is the right ground** — because if ability to pay is the right ground, the existing
progressive tax already implements the principle and my reform claim collapses. And that the
duty **survives the double-counting and stopping-point objections**, which I have answered
imperfectly and conceded are the weakest joints.

A reader who accepts both should accept the conclusion. A reader who rejects either has a
coherent alternative, and the model will price that alternative just as readily and just as
carefully. **That symmetry is not a weakness of the model. It is the reason the choice has to be
made in public rather than delegated to an optimiser.**

I hold this position. I do not claim the arithmetic proves it. What the arithmetic does — and
this is the part I would defend hardest — is show what the position costs, what it would raise,
and how much of it an early-retirement plan can quietly avoid while every column of the ledger
still reads correctly.

---

## 12. Relation to the rest of the repository

- **`EARLY_RETIREMENT.md` §6** — the four ledgers this chapter interprets, and the refusal to
  rank them. §8 of that document carries the CLI command that prints them.
- **`PHILOSOPHICAL_SOCIOLOGICAL_ASPECTS.MD` §5** — the Protestant ethic and the moralisation of
  work: this chapter is a companion argument about the moralisation of *education*.
- **`PHILOSOPHICAL_SOCIOLOGICAL_ASPECTS.MD` §6** — status, comparison and the limits of "enough".
  The benchmark argument in §7 above is that section's logic applied to a qualification.
- **`PHILOSOPHICAL_SOCIOLOGICAL_ASPECTS.MD` §8** — who receives the gain when AI buys back
  work-time. The same distributive question, one asset class over.
- **`src/early_retirement.rs`** — `ContributionPrinciple`, `Degree::declared_state_cost`,
  `contribution_ledger`, and the CLI's section 6.

## Further reading

Deliberately thin, and honestly labelled. No source consulted in this session measured the
public cost of a Swiss qualification, so **nothing here is cited as a source for those numbers**;
they are declared in the model and marked as such. The statutory citations that *are* sourced —
DBG Art. 38 on the separate taxation of capital benefits, StHG Art. 11 Abs. 3, BVG Art. 13a on
the three-step limit, and the AHV and BVG parameters — are listed with their vintages in
`EARLY_RETIREMENT.md` §4, which is where the provenance belongs.
