using TechTalk.SpecFlow;

namespace BeyondTrust.Automation.Windows.Sample
{
    [Binding]
    public class GnarlySteps
    {
        // 1. Plain literal — no captures
        [Given(@"caps lock is enabled")]
        public void GivenCapsLockIsEnabled() { }

        // 2. Single quoted-string capture
        [Given(@"a reader returns an error '(.*)'")]
        public void GivenAReaderReturnsAnError(string message) { }

        // 3. Numeric capture
        [Given(@"a time-limited approved JIT App Access request message for (\d+) days is displayed")]
        public void GivenJitDays(int days) { }

        // 4. Plain alternation (no quotes)
        [Given(@"a '(approved|pending|denied)' JIT App Access request message is displayed")]
        public void GivenJitMessage(string state) { }

        // 5. Multi-alternation in unquoted slot
        [Given(@"a (BeyondInsight|BeyondInsightTemp|BeyondInsightBackUp) (local user|local admin|machine) policy (does|does not|doesn't) exist with name '(.*)'")]
        public void GivenPolicy(string source, string type, string negation, string name) { }

        // 6. Optional trailing group
        [Given(@"I attempt to stop the '(.*)' service((?: as the user|))")]
        public void GivenStopService(string svc, string trailing) { }

        // 7. Named capture inside URL with comma-classes
        [Given(@"I build the repository '(https://github.com/[\w,\-,_]+/(?<repo>[\w,\-,_]+)\.git)' using Microsoft Build")]
        public void GivenBuildRepo(string url, string repo) { }

        // 8. Verbatim string with escaped quotes — note ""..."" in source = "..." at runtime
        [Given(@"""(.*)"" has an access level of ""(.*)"" for ""(.*)""")]
        public void GivenAccessLevel(string who, string level, string what) { }

        // 9. Non-verbatim string form (no @ prefix)
        [Given("groups from '(identity|pmc)' '(are|are not)' available for '(admin|user)'")]
        public void GivenGroups(string source, string state, string who) { }

        // 10. When-step with character class
        [When(@"I click '([^']*)' on the UAC popup")]
        public void WhenClick(string label) { }

        // 11. Then-step with multiple captures
        [Then(@"I ((?:fail to |))delete the '(.*)' key from '(.*)' in the '(.*)' hive and '(.*)' view")]
        public void ThenDelete(string fail, string key, string from, string hive, string view) { }

        // 12. Two attributes on one method (overload)
        [Given(@"I have a request for a (.*) minute JIT Admin session with reason '([^']*)'")]
        [Given(@"I have a request for a (.*) minute JIT Admin session with reason '([^']*)' as '(.*)'")]
        public void GivenJitRequest(string minutes, string reason, string asWho = null) { }
    }
}
