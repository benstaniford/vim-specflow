Feature: Smoke
	Covers a broad range of Client functionality out of a single config
	The config file is busy and confusing.  See comments for purpose of each set

#confirms a block rule with a drop down reason message
# Ignored on Insider builds due to SAMPLE-099
@NotInsider
@Smoke @SAMPLE-001
Scenario: Smoke - App Control - Block - With Reason
	Given I run the 'exe' resource 'calc.exe' from 'SystemDirectory'
	And I select 'I need to backup my computer' from the 'Please select a reason' drop down
	When I click 'Submit' on the elevation popup
	Then the process 'calc' with window 'Calculator' is not running
	And a local ECS event is created with EventId 116

#confirms run with the agent and adds an admin token
# Ignored on Insider builds due to SAMPLE-099
@NotInsider
@Not_ARM64
@Smoke @SAMPLE-002
Scenario: Smoke - App Control - Run With Acme
	Given I run 'DummyVbsScript.vbs' within 'Resources' using 'Run with Acme'
	When I click 'Yes' on the elevation popup
	Then the process 'wscript' with window 'Dummy VBS Script' is running
	And a local ECS event is created with EventId 107

#confirms a publisher match on a windows service
@Smoke_ServicePublisher @SAMPLE-003
Scenario: Smoke - App Matching - Positive - Publisher Matching
	Given the "Acme Sample Service" service is stopped
	When I start the 'Acme Sample Service' service as the user
	Then the 'Acme Sample Service' service status 'is' 'Running'
	And 1 local ECS event has been generated
	And the local ECS event 150 has the following details
		| Service.Name         | Service.DisplayName  |
		| Acme Sample Service  | Acme Sample Service  |

#confirms a negative match on untrusted owner
@Smoke @SAMPLE-004
Scenario: Smoke - App Matching - Negative - Trusted Owner
	Given I wait for 5 seconds
	And I take ownership of "\Resources\Testapplication.exe" as "BUILTIN\Administrators"
	When I run the 'exe' resource 'Testapplication.exe' from 'DeployDirectory'
	Then the process 'Testapplication' is running
	And a local ECS event is created with EventId 100

#confirms drop admin
# UAC will use the elevated token so that CreateProcessAsUser works but the agent will drop these rights
@Smoke @SAMPLE-005
Scenario: Smoke - App Privilege Management - Drop Admin
	When I run 'regedit.exe' with the following:
	| Field                  | value                          |
	| CommandLine            | c:\resources\dummy-reg-key.reg |
	| UseElevatedTokenForUAC | true                           |
	Then the process 'regedit' is running
	And the local user token for 'Admin' is applied to 'regedit.exe'
	And the process integrity for 'regedit' is 'Medium Mandatory Level'
	But the Administrator token is not applied to 'regedit.exe'

#confirms content control blocks access to a document via DummyContentHandler
@Smoke_ContentControl_Block @SAMPLE-006
Scenario: Smoke - Content Control - Block Access
	Given I open the content file 'Content.TXT.txt' from 'ContentDocuments' with the DummyContentHandler
	Then the DummyContentHandler was 'blocked from accessing' the file
	And a local ECS event is created with EventId 605

#confirms privilege management on a document via DummyContentHandler
@Smoke_ContentPrivMan @SAMPLE-007
Scenario: Smoke - Content Privilege Management - Add Admin
	Given I open the content file 'Content.DOCX.docx' from 'ContentDocuments' with the DummyContentHandler
	Then the DummyContentHandler was 'allowed to access' the file
	And a local ECS event is created with EventId 600
	And I see 'file.path' contains 'Content.DOCX.docx' in the local ECS event message body

#confirms filtering on user group
# Ignored on Insider builds due to SAMPLE-099
@NotInsider
@Smoke @SAMPLE-008
Scenario: Smoke - Filters - Groups - Specified Group Is Matched
	Given I run the 'msi' resource 'PowerGUI.3.6.0.21.msi' from 'DeployDirectory' as a domain user
	When I click 'OK' on the elevation popup
	Then the process 'msiexec' with window 'Quest PowerGUI® 3.6 Setup' is not running
	And a local ECS event is created with EventId 116

#negative case for child processes / precedence.  Confirms block rule on direct launch of child
# Ignored on Insider builds due to SAMPLE-099
@NotInsider
@Smoke @SAMPLE-009
Scenario: Smoke - Child Processes - Child Blocked On Direct Launch
	Given I run 'taskmgr.exe'
	When I click 'OK' on the elevation popup
	Then the process does not start
	And 1 local ECS event has been generated
	And the local ECS event 116 has the following details
		| file.path   |
		| taskmgr.exe |

#confirms exe's included in the client installer exist
@Smoke @SAMPLE-010
Scenario: Smoke - Installer - Check Bundled Applications Exist
	When the file 'C:\Program Files\Acme\Sample Client\SampleService.exe' 'does' exist
	Then the file 'C:\Program Files\Acme\Sample Client\Tray.exe' 'does' exist
	And the file 'C:\Program Files\Acme\Sample Client\ShellHelper.exe' 'does' exist
	And the file 'C:\Program Files\Acme\Sample Client\MessageHost.exe' 'does' exist
	And the file 'C:\Program Files\Acme\Sample Client\Mail.exe' 'does' exist
	And the file 'C:\Program Files\Acme\Sample Client\Capture.exe' 'does' exist
	And the file 'C:\Program Files\Acme\Sample Client\ActiveXInstall.exe' 'does' exist
