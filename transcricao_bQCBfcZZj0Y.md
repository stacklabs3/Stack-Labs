# Transcrição do vídeo bQCBfcZZj0Y

Boris Cherny, who made Claude Code,

landed two hundred fifty-nine PRs in a

month, all written by Claude.

Lauren Tan's team at xAI shipped eleven

thousand PRs, and their bots merge their

own bug fixes.

She says most of her tokens go to checking

the code.

You merge code you didn't read.

What if your AI could prove your code

works, in every case?

AI got really good at math proofs.

An Anthropic mathematician and Claude just

proved the Riemann hypothesis,

the most famous open problem in math.

It's seventeen million lines,

and computers checked every step.

Anthropic had Claude prove a hundred

twenty-three libraries don't crash.

Matt Pocock, who teaches TypeScript,

calls proofs essential.

Are proofs the missing piece of a software

factory like Lauren's?

So say you have a checkout in your app,

and you ask your agent to add a retry for

when the payment call times out.

The agent writes it, the tests are green,

and you say yes.

And then a customer gets charged twice.

So what happened is the first payment call

was slow, but it wasn't dead.

Your timer gave up on it and sent the

retry.

Then the first call landed anyway.

Then the retry landed.

And that's two charges on one card.

And the tests didn't catch it,

because each test tries one way things can

go, the one somebody picked,

and real traffic goes every way it can.

So how many ways are there?

So start with one ordinary thing.

The customer clicks pay,

and the payment call goes out.

Now a few things can happen next.

The provider answers fast,

or it answers slow, or the call fails,

or the network drops, or the customer

clicks pay again, or they close the tab.

So that's one step, and you already have

six branches.

And every one of those has its own things

that can happen next.

So take one of them, the slow answer.

From there, the reply comes back,

or your timer rings first.

Take the timer.

The retry goes out, and now there are two

calls out.

And the first one can land,

or the retry can land,

or both of them can, in either order.

And if both land, the customer is charged

twice.

So that's the bug from before,

and it's one path down a tree of

everything that can happen.

And the tree keeps splitting like that at

every step.

So if every step had only three things

that can happen next,

then ten steps is about fifty-nine

thousand paths, and twenty steps is about

three and a half billion.

And that's only which things happen.

There's also the order they happen in,

because the same things in a different

order is a different path.

So if ten things each happen once,

they can land in about 3.6

million orders, and twenty things in about

2.4 quintillion.

And if two customers pay at the same

moment, their steps get shuffled together

too.

So the branches and the orders multiply,

and it's close to endless.

And a test walks one path.

So five of them is five paths,

and your agent tests the paths it thought

of.

And taking a payment is simple.

The tree is what makes it hard.

So you want a check that covers the whole

tree, and that check exists.

You start with a rule,

and the rule is one sentence.

A customer is never charged twice for one

purchase.

Then the agent lists your checkout's steps

in a short file beside the code,

and that file is called a model.

Then a tool called a model checker walks

every branch of that tree.

And it can do that because a lot of paths

end up in the same place,

and it checks each place once.

When it gets to a path that breaks your

rule, it stops, and it hands you that path

back, step by step.

And you already know the fix.

It's an idempotency key,

so the payment provider treats the retry

as the same charge.

The agent adds it, the checker walks the

tree again, and no branch charges the card

twice.

And a proof goes further,

because it doesn't walk the tree at all.

It shows the rule holds at the start,

and that every single step keeps it

holding, whichever step it is.

So it holds at the end of every path,

in every order.

That's how math proves a thing for every

number.

It's true for the first one,

and each number passes it to the next.

So why weren't you doing this already?

Because it used to be really hard.

Amazon reported in twenty fifteen that a

check like this found a design bug in

DynamoDB, and the shortest path to that

bug was thirty-five steps deep.

You don't write that test by hand.

And there's an operating system kernel

called seL4, and proving about

eighty-seven hundred lines of its C code

took about twenty person-years.

So what changed is that the agent writes

these files now.

The model is written in a language called

TLA plus.

And Boris, the one with the two hundred

fifty-nine PRs, used two of these

languages, TLA plus and one called Lean.

And Lean is the language that Riemann

proof was released in.

And what he said about them was "I don't

know either language well,

but Claude is excellent at both." So

that's the check, and it has a name.

It's called formal verification.

So how do you get your agent to do that in

your repo?

Lauren Tan has a plugin called pstack,

which is the skills she uses every day,

and one of them is called create formal

verification skill.

So the first thing it does is pick what to

prove, and it picks that from your code.

It looks for a promise your code makes,

where breaking it hurts users,

and where your tests almost never hit the

case that breaks it.

And it goes for the one where a bug would

hurt the most, not the one that's easiest

to check.

Then it picks the tool,

and the question you're asking about your

code decides the tool.

If the question is about steps landing in

a bad order, which is what happened in the

checkout, it uses TLA plus,

and that's the default.

If the question is about every input to a

piece of code, it uses a checker that runs

on the real code, like Kani for Rust,

which checks the code for every possible

input.

And if there's no checker for your

language, it tests the real code with

generated inputs, and it tells you that's

what it did.

Then there's this line in the skill file:

"A check that you have never seen fail

proves nothing." So it breaks the code on

purpose first, and it watches the check go

red.

So in the checkout, it would take the

idempotency key back out,

and the checker has to find the double

charge again.

And when the check finds a problem,

the skill reports it.

It doesn't loosen the rule just to make it

pass.

And when it's done, it leaves a verify

skill in your repo.

That verify skill is one command,

and it knows the files it covers and the

commit it last checked,

so the next agent that works on that code

reruns it on every change.

And pstack has another skill called

statechart.

That one draws the states as a chart,

with the shortest path to the broken rule,

in plain words, so you don't have to read

TLA plus.

And there's a real case of this,

and it's close to the checkout with the

retry.

It's an open source project called

crashlens, and it has a button that asks a

paid AI model about a crash.

So the rule was one report,

at most one paid call.

And the checker broke it in two steps.

You press the button,

and you press it again before the answer

comes back.

And then a real server made two paid

calls.

And Lauren says she has "already found

several bugs" with it.

But look at what the checker checked.

It checked a model, and it checked a rule

that you wrote.

So is that enough?

That question is the most common reply

under every big post about this.

And there are two doubts in it.

The first doubt is about the rule.

A proof shows that the code keeps the rule

you wrote.

It doesn't show that the rule is right.

So if the rule is wrong,

the check still passes.

Go back to the checkout.

After the fix the rule passes.

But the tree still has a branch where the

card was charged once,

the retry timed out, and the purchase is

left pending.

The rule didn't say anything about that.

And it happened for real too,

in somebody else's payment model,

and it's in an open issue on pstack.

The rule there was that a purchase never

gets paid twice.

But there was a typo in the model,

so a purchase could never get paid.

And if a purchase can't get paid,

it can't get paid twice.

So the rule passed, and the typo was still

in there.

And that's what Lauren Tan,

who wrote the skill,

means when she says on camera:

"How do you know what it's proving?

It could be proving false is true." Now,

you already saw the first answer to this.

The skill has to see the check fail first.

And the second answer is you.

Your job moves to reading the rule,

and reading the list of what it leaves

out.

Lauren calls the rule the spec,

and she says "nothing can prove that your

spec is right, only you can judge." The

other doubt is about the model,

because the model is a file beside your

code, and your code can change without it.

Learn TLA plus says it in one line:

TLA plus tests designs,

not code.

So if somebody breaks the payment code and

leaves the model alone,

the check still passes,

because the model didn't know.

And the answer here is that verify skill,

because it records the commit and the

files it covers.

And a skill called maintain formal

verification skill tells you whether a

failure is the model falling behind or a

real bug in the code.

So what if the rule lived in the code,

and there was no model to fall behind?

That's what Victor Taelin built,

and it's called Bend 2,

and his talk calls it a proof language

that matches the speed of C.

You won't be using this one tonight,

but it's the same idea built into a

programming language.

In Bend 2 you write laws,

and the laws sit beside the code.

And Bend doesn't accept the code until the

agent proves that every law still holds.

So there's no separate model that can fall

behind.

We built a game in Bend 2 called Rooms

Within.

Every box on the floor is a whole room you

can walk into.

So you get rooms inside rooms.

And we wrote two laws for it.

A room can never end up inside itself.

And you can always walk back out.

The agent wrote the proof,

and it's about four hundred lines.

Then we made two copies of the game,

and we gave Claude one request in both.

"Let the player push a box into the room

they're standing in,

including the room they're inside." The

first copy has no laws,

so nothing in it says the rule.

Claude built it, and its checks passed.

And now the Archive room sits inside

itself, and inside itself again,

as deep as the screen can draw.

The second copy has the laws.

Claude tried it, and Bend's checker

rejected the change.

So Claude stopped.

It said the request as written breaks one

of the game's rules, and it isn't allowed

to change that rule.

Then it offered a version that keeps the

law, and we said yes.

So now you can push a box into another

room, or out into the room you're in.

But you can't push it into itself,

or into a room inside it.

The proof grew to about nine hundred

lines, and all the proofs check.

And Victor built Bend for agents that keep

improving code by themselves,

because, he says,

"Everything that you forget to test,

the AI will find a way to break." Now it

has limits, and he's the one who says

them.

You have to write the app in Bend,

so it can't check your TypeScript.

The part of Bend that checks the proofs is

proven.

The compiler, which turns Bend into a

program that runs,

isn't, and he says "There are many

compiler bugs." And he wrote that laws

"only protect what you remember to write,"

which is the same limit you saw with the

rule.

So what do you do with your app tonight?

So tonight you install pstack.

It lives in Cursor's plugin repo,

and Claude Code can install from it.

So in Claude Code you add Cursor's plugin

marketplace and then you install pstack.

And once it's installed,

it shows up in the Claude Code app too.

In Cursor it's add plugin pstack.

Then you run create formal verification

skill, and you aim it at the one promise

where a bug costs you the most.

That could be checkout,

notifications, sync, or a paid button.

And here's what it costs you.

It's in beta, it uses tokens,

TLA plus needs Java, and it does one rule

at a time.

You keep your tests.

This is for the one promise your tests

can't reach.

So, are proofs what the factory is

missing?

For that one promise,

yes.

You don't read every line anymore.

You read the one rule,

and the check goes through every case for

you.
