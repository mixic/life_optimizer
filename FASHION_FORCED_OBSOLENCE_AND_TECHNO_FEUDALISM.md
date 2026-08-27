# Fashion, Forced Obsolescence, and the Datacenter Behind the Screen

*A third companion to [`THEORY_OF_SPARING.md`](THEORY_OF_SPARING.md) and
[`HAPPYNESS_OR_FEAR_WORK_LIFE.md`](HAPPYNESS_OR_FEAR_WORK_LIFE.md).
Where `THEORY_OF_SPARING.md` asked whether a purchase is needed at all, this
chapter asks a narrower and more concrete question: when the product in
question is a smartphone, a PC, or a car, and "fashion" pressures you to
replace it every few years — is that replacement actually optional, or has it
quietly become semi-mandatory? And if the compute that justifies the upgrade
is increasingly running in someone else's datacenter, does the consumer even
need the new device at all?*

---

## 1. Two different kinds of obsolescence

`THEORY_OF_SPARING.md` §3 already distinguished manufactured *desire* (new
niches, new interfaces) from manufactured *necessity* (planned obsolescence).
Extending a product's ownership from five years to ten forces those two
mechanisms apart, because they respond completely differently to the decision
to hold out:

- **Fashion-driven obsolescence** is aesthetic and status-based: new clothes,
  a car redesign, a phone in a new color and shape. Nothing stops the product
  from functioning after the fashion cycle moves on. Holding out simply costs
  social signaling value (Veblen, already discussed in `THEORY_OF_SPARING.md`
  §3), not function.
- **Support-driven obsolescence** is infrastructural: a defined date after
  which security patches, OS updates, or app compatibility stop, regardless of
  whether the hardware still works perfectly. Holding out here risks real
  functional and security consequences, not just social ones.

The smartphone and PC cases you raise sit mostly in the second category, and
that is precisely what makes the "keep it for 10 years" question harder than
"wear last year's jacket for another winter" — the obsolescence is not just
manufactured taste, it is manufactured *and enforced* through a support
calendar the consumer does not control.

---

## 2. The Windows 11 / TPM 2.0 case, precisely

This is a genuinely good test case because the facts are concrete and
verifiable, and because it illustrates that "hardware obsolescence" is often
a **policy choice layered on top of** physical capability, not a pure hardware
limit.

- Windows 10 reached end of mainstream support on October 14, 2025. Microsoft
  offers a paid Extended Security Updates (ESU) program — reported around
  $30/year for consumers — as a bridge for people who do not upgrade.
- Windows 11 requires TPM 2.0, a minimum RAM/storage floor, and a CPU from an
  approved generation list (roughly 8th-generation Intel Core and newer, AMD
  Ryzen 2000-series and newer). Microsoft has stated TPM 2.0 is a
  "non-negotiable standard for the future of Windows," citing BitLocker,
  Secure Boot, and multi-factor-authentication features that depend on it.
- Critically, **most PCs sold in roughly the last five to eight years already
  physically contain a TPM 2.0 module — it is simply disabled in firmware by
  default.** A meaningful share of "incompatible" PCs are incompatible only
  because a BIOS setting was never turned on, not because the silicon is
  missing. This has been documented repeatedly by IT support communities and
  hardware forums throughout 2025.
- The CPU-generation cutoff is separately contested: critics note that many
  processors just below Microsoft's approved list can run Windows 11
  perfectly well once TPM/Secure Boot checks are bypassed (a registry key,
  or tools like Rufus, are commonly used for this), suggesting the boundary is
  drawn at least partly for support-simplification and security-marketing
  reasons, not solely hard technical necessity.

The consumer-relevant conclusion: a meaningful fraction of the "obsolescence"
in this specific case is **reversible with technical knowledge the average
consumer does not have or is not expected to use**, and Microsoft has publicly
declined to lower the requirement despite the resulting e-waste concern being
raised repeatedly in technology press and enthusiast communities throughout
2025. This is exactly the Section 7 dynamic from `Theory_of_Sparing.md` —
producers setting the support-window parameters that determine whether
"holding out" is a viable consumer strategy — applied to an operating system
rather than a device.

---

## 3. Smartphones: a genuinely improving, but still incomplete, picture

Smartphone software support windows have lengthened materially in recent
years — flagship devices from major manufacturers now commonly ship with
commitments in the six-to-seven-year range, compared to two-to-three years a
decade ago. This is real progress, driven partly by regulatory pressure (the
EU's ecodesign and "right to repair" rules, phased in since 2023, mandate
minimum spare-parts availability and security-update periods for phones sold
in the EU) and partly by manufacturers competing on longevity as a
sustainability claim.

Two things nonetheless limit how far this extends a phone's *usable*, not
just *supported*, life:

- **Non-replaceable batteries.** A sealed battery that cannot be
  user-replaced converts a purely software-supported device into one with a
  hard physical failure point after a few hundred charge cycles — independent
  of whether the OS is still updated. This is the clearest example in this
  whole chapter of `Theory_of_Sparing.md` §3's planned-obsolescence critique
  (Bulow, 1986) applied to a specific, physical design choice.
- **App-ecosystem minimum requirements.** Even a phone still receiving
  security patches can be pushed into replacement by individual apps (banking,
  authentication, transit) raising their own minimum supported OS version
  faster than the phone's official support window allows — a second,
  decentralized obsolescence mechanism layered on top of the manufacturer's
  own support calendar, and one no single company is fully accountable for.

---

## 4. Is ten years actually rational? A cost-benefit reframing

`Theory_of_Sparing.md` §2 proposed a decision rule: effective value is utility
delivered per franc spent, weighted by realistic utilization. Applied here:

- **Realistic threat model for a typical consumer.** The security features
  TPM 2.0 and similar hardware-rooted protections defend against are real, but
  are disproportionately relevant to targeted attacks, enterprise
  environments, and high-value accounts — not the median personal-use
  scenario. Several of the sources documenting the TPM 2.0 rollout explicitly
  raise this point: the security case is strong for institutions and weaker,
  though not zero, for casual personal use.
- **The ESU bridge changes the arithmetic.** At roughly $30/year, several
  years of extended support cost a small fraction of a new device, which is a
  textbook case of the `Theory_of_Sparing.md` §2 "effective value" comparison:
  a low-cost bridge against a device with years of remaining physical life
  can easily out-perform a full replacement in utility-per-franc, particularly
  for a household already resource-constrained by the essential-cost pressure
  discussed in `Theory_of_Sparing.md` §7.
- **The opportunity cost is the part usually left out.** A new laptop and
  phone every five years, for a household of several people, over a decade,
  is a material sum — and `Theory_of_Sparing.md`'s formalization treats that
  sum as fungible: it can just as easily fund travel, shared experience, or
  savings as it can fund the next device cycle.

This connects directly to a well-documented finding in happiness research:
Van Boven & Gilovich's work on experiential versus material purchases (2003,
and a substantial body of subsequent replication) finds that spending on
experiences — travel being the paradigmatic example — tends to produce more
durable reported life satisfaction than spending on equivalent-value material
goods, partly because experiences resist the hedonic adaptation that quickly
normalizes a new device back to "unremarkable." A household reallocating a
tech-refresh budget toward travel is not simply trading one discretionary
category for another of equal standing — the research suggests it is trading
toward the category with better-documented long-run wellbeing return.

None of this means new hardware is never justified — a genuinely obsolete,
failing, or performance-limiting device is a real cost too. The point is that
the *default* five-year cycle is rarely actually re-examined against this
comparison; it is followed because it is the fashion-plus-support-window norm,
not because it was compared against the alternative.

---

## 5. The techno-feudal asymmetry: whose refresh cycle is this, really?

Here the argument connects directly to
`HAPPYNESS_OR_FEAR_WORK_LIFE.md` §5's account of Varoufakis's
technofeudalism thesis, and it is worth being precise about scale, because the
asymmetry is large and current.

The four largest hyperscalers (Amazon, Microsoft, Alphabet/Google, and Meta)
have guided toward a combined capital expenditure of roughly **$725 billion
in 2026** — up around 77% from approximately $410 billion in 2025 — with the
large majority directed at AI datacenters, GPU clusters, and the power
infrastructure to run them. A meaningful share of this is now debt-financed:
industry reporting describes over $100 billion in new debt issuance across
hyperscalers in 2025 alone, with projections of substantially more over the
following years. This is not a steady-state cost; it is an accelerating,
largely debt-fueled capital cycle, and GPU generations underlying it (roughly
a two-year cadence in recent Nvidia architecture releases) age out of
frontier-competitive status faster than almost any consumer hardware category
discussed above.

The asymmetry with the household case in Sections 1–4 is stark:

- **A consumer deciding whether to replace a PC every 5 or 10 years is making
  a bounded, personally-financed capital decision** they can defer, reverse
  (via ESU-style bridges), or opt out of.
- **A consumer paying a monthly AI subscription or per-token API fee has no
  equivalent lever over the underlying infrastructure renewal decision at
  all.** They do not vote on, finance directly, or control when a datacenter
  is rebuilt around a newer GPU generation — they simply pay a recurring rent
  whose price reflects, among other things, the amortized cost of a capital
  cycle happening on a schedule set entirely by the platform owner.

This is precisely the **cloud rent** relationship `HAPPYNESS_OR_FEAR_WORK_LIFE.md`
§5 already described: the consumer's household capex decision (keep the old
PC or not) is now increasingly decoupled from — and largely irrelevant to —
the far larger, faster-cycling capex decision (rebuild the datacenter or not)
that actually determines the cost and capability of the AI services they
subscribe to. Sparing at the household hardware level does not touch this
layer at all; it operates entirely downstream of it.

---

## 6. Does the consumer even need the powerful PC? A genuine paradigm question

This is the sharpest question in your framing, and it cuts in a
counterintuitive direction: **the more that compute-intensive work (LLM
inference, AI-assisted coding, image and video generation) moves to cloud
subscription and API-based services, the less local hardware performance
actually matters** — which is, in principle, an argument *for* extending
device lifecycles, not against it. A thin, older laptop that can run a
browser and a terminal is functionally sufficient to drive a cloud-hosted
coding agent or chat interface that does the heavy computation remotely. In
this sense, the shift toward cloud AI could reduce the pressure to buy new
*local* hardware precisely as the underlying compute need grows.

But this apparent relief comes with a structural trade, not a free lunch:

- **Capex is replaced by opex, and ownership is replaced by rent.** A
  household that stops needing to buy a new powerful PC every five years, but
  starts paying a recurring subscription or per-token fee for cloud AI
  compute, has not necessarily reduced its total long-run payments to the
  technology sector — it has shifted them from a depreciating owned asset to
  a non-owned, revocable, price-adjustable service. This is the household-
  level mirror of Varoufakis's core claim: value capture shifts from a
  competitive goods market (you own the laptop, competing manufacturers set
  its price once) to a rentier relationship (you never own the compute,
  and the rent-setter can change price, throttle access, or discontinue the
  service).
- **The dependency is less visible, which makes it harder to spare against.**
  A five-year-old laptop is a visible, self-contained decision the household
  can evaluate directly (Sections 1–4 above). A monthly AI subscription
  embedded across several tools and workflows is a diffuse, recurring, and
  easily overlooked cost — closer to `Theory_of_Sparing.md` §7b's
  "quasi-inelastic" category than to the elastic, sparing-eligible category,
  precisely because switching away from an AI tool that has become embedded
  in daily workflow carries its own switching cost.
- **There is a live counter-trend worth naming.** Open-weight models and
  increasingly capable on-device neural processing units (NPUs) in recent
  consumer hardware represent an explicit attempt to keep some AI capability
  local and owned rather than rented — a genuine, if currently partial,
  alternative to full cloud dependency. Whether this remains a meaningful
  counterweight to the hyperscaler capex cycle described in Section 5, or
  ends up as a minority niche alongside an overwhelmingly cloud-rent-based
  default, is a genuinely open question this document does not attempt to
  settle.

The honest framing of the paradigm shift, then, is not "the consumer needs
less hardware, so this is good news for sparing." It is: **the locus of
capital-cycle dependency is moving from a decision the household makes and
can defer (hardware replacement) to one it does not make and cannot defer
(paying for access to someone else's accelerating capex cycle)** — which may
reduce e-waste and household hardware spend while simultaneously deepening
the technofeudal rent relationship this document series has already
identified as the more structurally important trend.

---

## 7. The hidden cost of the cloud: memory prices, water, and exponential energy

Section 5 established that hyperscalers face an accelerating, largely
debt-financed capital cycle the consumer has no lever over. It is worth being
precise about three further costs embedded in that cycle, because they answer
your question directly: the memory price shock is not something only
consumers feel, and the resource cost of running the resulting infrastructure
extends well beyond money.

### 7a. Hyperscalers pay the same inflated component prices consumers do — at far larger scale

The DRAM/NAND price shock already discussed in `Theory_of_Sparing.md` §6a does
not spare the hyperscalers; if anything it hits them harder, because a
single AI training cluster can require memory capacity equivalent to many
thousands of consumer devices, and much of it is high-bandwidth memory (HBM)
specifically bid up by AI accelerator demand. Every datacenter "renovation" —
the roughly two-year cadence at which frontier GPU generations are refreshed —
now competes for the same constrained memory supply that has driven consumer
component prices sharply higher since 2024. This means the ~$725 billion in
2026 hyperscaler capex discussed in Section 5 is not a fixed sum being spent
more aggressively; it is a sum that itself has to absorb a genuine, currently
ongoing input-cost shock, on top of the underlying growth in raw compute
purchased.

### 7b. Water: a real, measurable, and geographically concentrated cost

Datacenter cooling — whether through evaporative cooling towers or newer
liquid/immersion methods — consumes water both directly (evaporated during
cooling) and indirectly (through the water intensity of the electricity that
powers the facility and its chillers). The scale is now large enough to be
a genuine infrastructure constraint, not a rounding error:

- U.S. datacenter water consumption, driven substantially by AI expansion,
  was estimated to approach nearly one trillion liters annually by 2025.
- Global AI-related water withdrawals have been projected to reach
  4.2–6.6 billion cubic meters annually by 2027 — a range researchers have
  compared to four to six times Denmark's total annual water consumption.
- Training a single large frontier model has been estimated, in at least one
  widely cited study of GPT-3-scale training in Microsoft's U.S. facilities,
  to consume several million liters of water for on-site cooling alone —
  before counting the indirect water embedded in the electricity used.
- Up to 85% of the water used in evaporative cooling does not return to the
  local water supply; it evaporates. This is precisely why datacenter
  expansion has become contentious in arid and semi-arid regions, where new
  facilities compete directly with municipal drinking water and agricultural
  irrigation — a tension reported in multiple U.S. states and, internationally,
  in several drought-affected regions where datacenter proposals have faced
  local opposition on exactly these grounds.

This is a cost with no equivalent in the consumer-hardware side of this
chapter's argument: a five-year-old laptop draws no water at all once
purchased. The datacenter behind a cloud AI subscription draws water
continuously, for as long as it runs — which is one further respect in which
Section 6's "capex-to-opex" shift is not resource-neutral, only differently
distributed.

### 7c. Energy: growth that is genuinely exponential, not merely large

The International Energy Agency's 2025 *Energy and AI* report estimated
global datacenter electricity consumption at roughly 415 terawatt-hours in
2024, and projected this to **more than double by 2030** — a growth rate
substantially faster than overall electricity demand. In the United States
specifically, datacenters have grown from about 1.9% of national electricity
consumption in 2018 toward a share some projections place as high as 12% by
2028. AI-specific U.S. datacenter emissions have been projected to add
24–44 million additional metric tons of CO2-equivalent annually by 2030,
depending on the carbon intensity of the electricity supplying them — which
is itself why hyperscalers have increasingly turned to direct power-purchase
agreements with nuclear plants (Microsoft's agreement to help revive the
Three Mile Island plant being one widely reported example) rather than
relying solely on the existing grid.

This growth curve is the infrastructure-scale mirror of the household
question in Sections 1–4: just as a consumer rarely stops to ask whether a
five-year replacement cycle is actually justified by the marginal benefit
delivered, the AI infrastructure buildout rarely surfaces, in a way visible to
the end user, what a given query, image generation, or coding-agent session
actually cost in water and electricity to produce — the resource cost is real,
growing exponentially, and almost entirely invisible at the point of use.

---

## 8. Can this be done more efficiently? Small models, photonics, and the Jevons paradox

This is the most important question in my framing, because it asks whether
the resource cost documented in Section 7 is an inherent property of AI
compute, or a byproduct of a particular, currently dominant engineering
strategy — and, crucially, whether efficiency gains would actually *reduce*
the footprint of the system as a whole, or simply be absorbed into more of it.

### 8a. Yes — meaningfully more efficient models already exist and are improving quickly

Contrary to the assumption that AI capability requires ever-larger models,
there is an active and fast-moving research program aimed at exactly the
opposite: extracting more capability per unit of compute, memory, and energy.

- **Quantization** reduces the numerical precision used to store and compute
  a model's parameters (for example from 16-bit to 4-bit representations),
  cutting memory footprint and often substantially speeding up inference,
  typically with a modest, task-dependent accuracy cost.
- **Knowledge distillation** trains a much smaller "student" model to
  reproduce the behavior of a larger "teacher" model. A widely discussed 2025
  example: distilled versions of a large reasoning model (DeepSeek-R1),
  compressed down to a few billion parameters, achieved strong performance on
  demanding math and coding benchmarks — in some reported cases outperforming
  substantially larger general-purpose models on those specific tasks — at a
  small fraction of the parameter count and inference cost.
- **Pruning** removes redundant parameters entirely, and combined
  pruning-distillation-quantization pipelines have been shown in recent
  research to preserve most of a model's capability at a fraction of its
  original footprint.
- **Small language models (SLMs)**, generally under roughly 10 billion
  parameters and often specialized for a narrower task set, are an
  increasingly viable production alternative to frontier-scale general models
  for a large share of real-world use cases — the honest industry assessment
  emerging through 2025–2026 is that most practical tasks do not require
  frontier-model-level capability at all.

This directly answers part of your question: **a task that only needs a
correct classification, a short lookup, or a routine transformation, but is
routed through a frontier-scale model anyway, is an efficiency failure at
the task-allocation level** — the computational and resource equivalent of
using a full datacenter query to do what a much smaller, specialized model
(or, for the simplest cases, a plain deterministic script) could do at a
tiny fraction of the energy and water cost. Right-sizing the model to the
task, not just shrinking models in general, is where a meaningful share of
the "what do we actually get for this" gap sits.

### 8b. Genuinely new hardware: computing with light

Your instinct about a fundamentally different computer architecture is not
speculative — it is an active, funded, commercially emerging field. Photonic
(optical) computing performs the core operation of a neural network — large
matrix multiplications — using light rather than electrons: passing an
optical signal through a mesh of interferometers and phase shifters can
perform many multiply-accumulate operations essentially in parallel, in
roughly the time light takes to cross the chip, compared to many electronic
clock cycles for the equivalent operation.

- Companies including Lightmatter, Lightelligence, and Q.ANT have built
  photonic processors specifically for AI inference, with peer-reviewed
  results published in *Nature* in April 2025. Lightmatter's CEO has publicly
  claimed up to a 10x energy-efficiency improvement over an NVIDIA A100 GPU
  for inference workloads specifically — a claim specific to inference, not a
  general replacement claim for all computing.
- A related but distinct application — **photonic interconnects** (moving
  data between chips using light rather than copper, as opposed to computing
  with light) — is commercially further along: Marvell's acquisition of
  photonic-interconnect company Ayar Labs for $3.25 billion in December 2025
  is one concrete signal of how seriously the industry is investing in this
  adjacent optical technology, even where full optical computation is not yet
  mature.
- The honest current state, as of 2025–2026 industry assessment, is that
  photonic compute chips are **complementary accelerators for specific,
  matrix-multiplication-heavy inference workloads, not a general replacement**
  for electronic computing — the software ecosystem is far less mature than
  existing GPU tooling, and photonics handles the "linear algebra" portion of
  a workload while conventional electronics still manages control logic,
  memory, and the non-linear operations neural networks also require.

This is a genuinely promising efficiency lever, but it is not yet a
near-term solution to the energy and water figures in Section 7 — it is a
real research and early-commercial trajectory worth tracking, not a
currently available fix.

### 8c. The Jevons paradox: why efficiency alone may not shrink the footprint

Here is the central strategic question your framing raises, and it deserves
a direct, historically grounded answer: **if AI compute becomes dramatically
more efficient, will total resource consumption actually fall?**

Economic history gives a specific, well-documented reason to doubt it. William
Stanley Jevons, writing in *The Coal Question* (1865), observed that
improvements in the efficiency of steam engines did not reduce Britain's total
coal consumption — they increased it, because a more efficient engine made coal
power profitable for a much wider range of uses than before. This
**rebound effect**, since generalized well beyond coal, is one of the most
robust findings in the economics of technological efficiency: making a
resource cheaper to use per unit of output typically *expands* the range of
economically viable uses for it faster than the per-unit saving reduces
consumption, producing a net increase in total resource use rather than a
decrease.

Applied here: if a distilled, quantized, or photonic-accelerated model can
answer a query at one-tenth the energy cost of today's frontier model, the
historically likely outcome is not that hyperscalers scale back datacenter
construction by a proportional amount. It is that the same capital is
redeployed toward **ten times as many queries, deeper agentic tool-calling
loops, longer context windows, more redundant retries, and entirely new
product categories** that were not economical at the old cost per query —
precisely mirroring how cheaper coal expanded, rather than shrank, its total
use in the 19th century. The 2026 capex figures in Section 5 are already
consistent with this pattern: even as model efficiency has measurably
improved year over year throughout the current AI buildout, aggregate
capital expenditure and energy demand have continued to grow, not shrink,
because efficiency gains have so far been reinvested into scale rather than
banked as reduced total footprint.

### 8d. What this means for the Cloudalist model specifically

This has a direct and somewhat counterintuitive implication for Varoufakis's
technofeudalism framing, discussed in Section 5 and in
`HAPPYNESS_OR_FEAR_WORK_LIFE.md` §5. A cloudalist's revenue depends on
*usage* of the platform, not on the efficiency of any individual query. A more
efficient model does not threaten the cloud-rent business model at all — if
anything, it strengthens it, by lowering the marginal cost of serving each
additional query and therefore widening the profitable range of use cases the
platform can rent access to, exactly as Jevons's coal-efficiency mechanism
predicts. Efficiency research is genuinely valuable — it is real, well
grounded, and worth pursuing — but it is not, by itself, a force that
disrupts the concentration of compute ownership Section 5 describes. It is,
if anything, a force that makes the underlying infrastructure cheaper to
operate and therefore *more* profitable to own, without changing who owns it.

The one place efficiency *does* meaningfully shift power, rather than just
lowering cost, is the "right-sizing" point in Section 8a taken to its logical
conclusion: a small, efficient, **locally run** model that handles the bulk
of routine tasks on a consumer's own device removes that workload from the
rented infrastructure entirely, rather than merely making the rented
infrastructure cheaper to run. This is the same distinction Section 6 already
drew between owned compute and rented compute — efficiency gains captured by
the cloudalist reinforce the rent relationship; efficiency gains captured by
the consumer, in the form of a genuinely capable model running on their own
hardware, are the one lever that reduces dependency on the rent relationship
itself, rather than just making it cheaper to sustain.

---

## 9. Extending the Life Optimizer consumption model

`Theory_of_Sparing.md` §8 already proposed splitting consumption into
inelastic, quasi-inelastic, and elastic tiers. This chapter's argument fits
directly into that structure and suggests one further refinement: separating
**owned-durable spending** (the device itself) from **recurring platform
rent** (the subscription that increasingly substitutes for local compute),
since the two behave very differently as elasticity categories and respond to
completely different sparing levers.

$$
Q_t = Q_t^{\text{device}} + Q_t^{\text{platform rent}}
$$

where $Q_t^{\text{device}}$ is amortized hardware replacement cost (directly
reducible by extending the replacement cycle from 5 to 10 years, as this
chapter argues is often underexplored) and $Q_t^{\text{platform rent}}$ is
recurring cloud/AI subscription and per-token spend (which Section 6 argues is
*not* reduced by extending device lifecycles, and may in fact be growing as a
substitute for them).

A device replacement cycle parameter follows naturally:

$$
Q_t^{\text{device}} = \frac{P_{\text{device}}}{\tau_{\text{replacement}}}
$$

where $\tau_{\text{replacement}}$ is the chosen ownership horizon in years.
Doubling $\tau_{\text{replacement}}$ from 5 to 10 halves this term directly —
a concrete, quantifiable version of the "keep it longer" strategy this chapter
opened with, and one the CLI could expose as `--device-replacement-years`
alongside the `--sparing-ratio` and `--quasi-inelastic-share` flags already
proposed in `FutureWork.md` §5.1.

---

## 10. Summary

- Fashion-driven obsolescence (aesthetic, status-based) and support-driven
  obsolescence (software/security lifecycle cutoffs) are distinct mechanisms
  that respond very differently to a "keep it longer" strategy — the second
  is the one that actually constrains extending PC and phone ownership from
  five to ten years.
- The Windows 11 / TPM 2.0 case shows that a meaningful share of "hardware
  obsolescence" is a support-policy choice layered on top of hardware that is
  often still physically capable — Microsoft has stated the requirement is
  non-negotiable despite the resulting e-waste concern being raised
  repeatedly since 2025.
- Smartphone support windows have genuinely lengthened (helped by EU
  right-to-repair regulation since 2023), but non-replaceable batteries and
  app-ecosystem minimum-version creep still limit how far official software
  support translates into extended real-world usable life.
- A cost-benefit reframing using the `THEORY_OF_SPARING.md` effective-value
  formula, combined with experiential-versus-material happiness research (Van
  Boven & Gilovich), suggests that redirecting a five-year tech-refresh
  budget toward travel and experience is not merely an equal trade but one
  with a documented long-run wellbeing advantage.
- The four largest hyperscalers are guiding toward roughly $725 billion in
  combined 2026 capital expenditure — a debt-fueled, rapidly accelerating
  capital cycle the consumer has no equivalent lever over, unlike their own
  household hardware-replacement decision.
- Moving compute to the cloud could, in principle, reduce the need for
  powerful local hardware — but this substitutes owned, deferrable capex for
  non-owned, non-deferrable platform rent, deepening rather than resolving
  the technofeudal dependency already identified in
  `HAPPYNESS_OR_FEAR_WORK_LIFE` §5.
- The memory price shock hits hyperscalers at far greater absolute scale than
  consumers, and the resulting datacenter buildout carries real, measurable,
  and exponentially growing water and energy costs — global datacenter
  electricity demand is projected to more than double between 2024 and 2030,
  and AI-driven water withdrawals are already measured in billions of cubic
  meters annually.
- Genuinely more efficient AI — quantized and distilled small models,
  right-sized to the task, and eventually photonic hardware for
  matrix-heavy inference — is real, active, and improving quickly. But the
  Jevons paradox gives good historical reason to expect efficiency gains to be
  reinvested into more usage rather than banked as reduced total resource
  consumption, which would leave the cloudalist rent relationship intact, or
  even strengthened, rather than disrupted — unless the efficiency is captured
  by the consumer, in the form of locally run capability, rather than by the
  platform.
- The consumption model proposed in `THEORY_OF_SPARING.md` §8 can be extended
  with a device-replacement-horizon parameter and a separate platform-rent
  term, making this trade-off an explicit, adjustable input rather than an
  implicit five-year default.

---

## Further reading

- Jeremy Bulow, "An Economic Theory of Planned Obsolescence" (1986) — already
  cited in `THEORY_OF_SPARING.md` §3, directly applicable to sealed-battery
  smartphone design
- Leaf Van Boven & Thomas Gilovich, "To Do or to Have? That Is the Question"
  (*Journal of Personality and Social Psychology*, 2003) — the foundational
  experiential-versus-material purchase happiness research
- Microsoft, official Windows 11 minimum system requirements and TPM 2.0
  documentation (support.microsoft.com), and Microsoft's public statements
  reiterating TPM 2.0 as non-negotiable for future Windows versions
- European Commission, ecodesign and right-to-repair requirements for
  smartphones and tablets (phased in from 2023) — minimum spare-parts
  availability and security-update-period mandates
- Industry earnings and analyst reporting on 2025–2026 hyperscaler capital
  expenditure (Goldman Sachs analysis; company earnings calls for Amazon,
  Microsoft, Alphabet, and Meta) — the ~$725 billion 2026 combined AI capex
  figure and associated debt-financing reporting cited in Section 5
- International Energy Agency, *Energy and AI* (2025) — global datacenter
  electricity demand estimates and projections cited in Section 7c
- Lawrence Berkeley National Laboratory, *2024 United States Data Center
  Energy Usage Report* (Shehabi et al.) — U.S. datacenter electricity-share
  figures cited in Section 7c
- Li et al. (2023) and related 2025–2026 research on AI water footprint —
  the GPT-3 training water estimate and global 2027 AI water-withdrawal
  projections cited in Section 7b
- William Stanley Jevons, *The Coal Question* (1865) — the original
  formulation of the rebound effect underlying Section 8c's argument
- Recent (2024–2026) research on LLM compression: quantization (GPTQ,
  SmoothQuant, AWQ), knowledge distillation (DeepSeek-R1 distilled model
  family), and pruning-distillation-quantization pipelines — cited in
  Section 8a
- Reporting on photonic AI accelerators (Lightmatter, Lightelligence, Q.ANT,
  Celestial AI, Ayar Labs) and *Nature* publications on photonic processors
  (April 2025) — cited in Section 8b
- Cross-reference: [`THEORY_OF_SPARING.md`](THEORY_OF_SPARING.md) §2–3, §6a, §7–8;
  [`HAPPYNESS_OR_FEAR_WORK_LIFE.md`](HAPPYNESS_OR_FEAR_WORK_LIFE.md)
  §5; [`FutureWork.md`](FutureWork.md) §5.1
