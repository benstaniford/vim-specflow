// Curated bindings extracted verbatim from the real EPM SpecFlow corpus,
// containing exactly (and only) the bindings exercised by Smoke.feature.
//
// This file is the hermetic equivalent of the live corpus — when this test
// file plus Smoke.feature resolve cleanly, the helper is correct for every
// shape of pattern that appears in the smoke suite.

using TechTalk.SpecFlow;

namespace BeyondTrust.Automation.Windows.Smoke.Bindings
{
    [Binding]
    public class SmokeBindings
    {
        // --- RunProcess ---------------------------------------------------
        [Given(@"I run the '(.*)' resource '(.*)' from '([^']*)'")]
        public void GivenIRunResource(string kind, string name, string from) { }

        [When(@"I run the '(.*)' resource '(.*)' from '([^']*)'")]
        public void WhenIRunResource(string kind, string name, string from) { }

        [Given(@"I run the '(.*)' resource '(.*)' from '(.*)' as a (local|domain) (user|admin)")]
        public void GivenIRunResourceAs(string k, string n, string f, string scope, string who) { }

        [Given(@"I run '([^']*)'")]
        public void GivenIRunBare(string what) { }

        [Given(@"I run '([^']*)' with the following:")]
        [When(@"I run '([^']*)' with the following:")]
        public void IRunWithTable(string what, Table table) { }

        [Given(@"I open the content file '([^']*)' from '([^']*)' with the DummyContentHandler")]
        public void GivenOpenContent(string file, string from) { }

        // --- WindowsExplorer ---------------------------------------------
        [Given(@"I run '(.*)' within '(.*)' using '(.*)'((?: from the original folder|))")]
        public void GivenIRunWithin(string what, string within, string using_, string trailing) { }

        // --- DefendpointNotification -------------------------------------
        [Given(@"I select '(.*)' from the '(.*)' drop down")]
        public void GivenISelect(string item, string label) { }

        [When(@"I click '(.*)' on the elevation popup")]
        public void WhenIClick(string label) { }

        [Given(@"I wait for (.*) (seconds|minutes)")]
        public void GivenIWait(string amount, string unit) { }

        // --- Services ----------------------------------------------------
        [Given(@"the ""(.*)"" service is stopped")]
        public void GivenServiceStopped(string svc) { }

        [When(@"I start the '(.*)' service((?: as the user|))")]
        public void WhenIStartService(string svc, string trailing) { }

        [Then(@"the '(.*)' service status '(is|is not|isn't)' '(.*)'")]
        public void ThenServiceStatus(string svc, string negation, string status) { }

        // --- ProcessDetails ----------------------------------------------
        [Then(@"the process '([^']*)' with window '([^']*)' (.*) running")]
        public void ThenProcessWithWindow(string proc, string window, string state) { }

        [Then(@"the process '([^']*)' (is|is not|isn't) running")]
        public void ThenProcessRunning(string proc, string state) { }

        [Then(@"the (domain|local) user token for '(.*)' (.*) applied to '([^']*)'")]
        public void ThenUserTokenApplied(string scope, string who, string state, string proc) { }

        [Then(@"the process integrity for '([^']*)' is '(.*)'")]
        public void ThenProcessIntegrity(string proc, string level) { }

        [Then(@"the Administrator token (.*) applied to '([^']*)'")]
        public void ThenAdminTokenApplied(string state, string proc) { }

        [Then(@"the process (has started|does not start|is still waiting for our messagehost)")]
        public void ThenProcessStart(string state) { }

        // --- DummyContentHandler -----------------------------------------
        [Then(@"the DummyContentHandler was '(allowed to access|blocked from accessing)' the file")]
        public void ThenDummyAccess(string verdict) { }

        // --- EventLogs ---------------------------------------------------
        [Then(@"a local ECS event (is|is not|isn't) created with EventId (.*)")]
        public void ThenLocalEcsCreated(string negation, string eventId) { }

        [Then(@"(\d+) local ECS event has been generated")]
        public void ThenEcsCount(string count) { }

        [Then(@"the local ECS event (.*) has the following details")]
        public void ThenEcsDetails(string id, Table table) { }

        [Then(@"I see '(.*)' contains '(.*)' in the local ECS event message body")]
        public void ThenEcsBodyContains(string key, string value) { }

        // --- File ownership / existence ----------------------------------
        [Given(@"I take ownership of ""([^""]*)"" as ""(.*)""")]
        public void GivenTakeOwnership(string file, string asWho) { }

        [When(@"the file '(.*)' '(does|does not)' exist")]
        [Then(@"the file '(.*)' '(does|does not)' exist")]
        public void FileExistence(string file, string state) { }
    }
}
