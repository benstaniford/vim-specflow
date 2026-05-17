Feature: Smoke
	Covers a broad range of Client functionality out of a single config
	The config file is busy and confusing.  See comments for purpose of each set

#confirms a block rule with a drop down reason message
# Ignored on Smokes Insider due to RAD-3022
@NotInsider
@Smoke @EPM-17971
Scenario: Smoke - App Control - Block - With Reason
	Given I run the 'exe' resource 'calc.exe' from 'SystemDirectory'
	And I select 'I need to backup my computer' from the 'Please select a reason' drop down
	When I click 'Submit' on the elevation popup
	Then the process 'calc' with window 'Calculator' is not running
	And a local ECS event is created with EventId 116

#confirms run with defendpoint and adds an admin token
# Ignored on Smokes Insider due to RAD-3022
@NotInsider
@Not_ARM64
@Smoke @EPM-17967
Scenario: Smoke - App Control - Run With Defendpoint
	Given I run 'DummyVbsScript.vbs' within 'Resources' using 'Run with Defendpoint'
	When I click 'Yes' on the elevation popup
	Then the process 'wscript' with window 'Dummy VBS Script' is running
	And a local ECS event is created with EventId 107

#confirms a publisher match on a windows service
@Smoke_ServicePublisher @EPM-17969
Scenario: Smoke - App Matching - Positive - Publisher Matching
	Given the "Avecto QA Test Windows Service" service is stopped
	When I start the 'Avecto QA Test Windows Service' service as the user
	Then the 'Avecto QA Test Windows Service' service status 'is' 'Running'
	And 1 local ECS event has been generated
	And the local ECS event 150 has the following details
		| EPMWinMac.ServiceControl.Service.Name | EPMWinMac.ServiceControl.Service.DisplayName |
		| Avecto QA Test Windows Service        | Avecto QA Test Windows Service               |

#confirms a negative match on untrusted owner
@Smoke @EPM-17973
Scenario: Smoke - App Matching - Negative - Trusted Owner
	Given I wait for 5 seconds
	And I take ownership of "\Resources\Testapplication.exe" as "BUILTIN\Administrators"
	When I run the 'exe' resource 'Testapplication.exe' from 'DeployDirectory'
	Then the process 'Testapplication' is running
	And a local ECS event is created with EventId 100

#confirms drop admin
# UAC will use the elevated token so that CreateProcessAsUser works but EPM-W will drop these rights
@Smoke @EPM-17968
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
@Smoke_ContentControl_Block @EPM-17966
Scenario: Smoke - Content Control - Block Access
	Given I open the content file 'Content.TXT.txt' from 'ContentDocuments' with the DummyContentHandler
	Then the DummyContentHandler was 'blocked from accessing' the file
	And a local ECS event is created with EventId 605

#confirms privilege management on a document via DummyContentHandler
@Smoke_ContentPrivMan @EPM-17972
Scenario: Smoke - Content Privilege Management - Add Admin
	Given I open the content file 'Content.DOCX.docx' from 'ContentDocuments' with the DummyContentHandler
	Then the DummyContentHandler was 'allowed to access' the file
	And a local ECS event is created with EventId 600
	And I see 'file.path' contains 'Content.DOCX.docx' in the local ECS event message body

#confirms filtering on user group
# Ignored on Smokes Insider due to RAD-3022
@NotInsider
@Smoke @EPM-17970
Scenario: Smoke - Filters - Groups - Specified Group Is Matched
	Given I run the 'msi' resource 'PowerGUI.3.6.0.21.msi' from 'DeployDirectory' as a domain user
	When I click 'OK' on the elevation popup
	Then the process 'msiexec' with window 'Quest PowerGUI® 3.6 Setup' is not running
	And a local ECS event is created with EventId 116

#negative case for child processes / precedence.  Confirms block rule on direct launch of child
# Ignored on Smokes Insider due to RAD-3022
@NotInsider
@Smoke @EPM-17965
Scenario: Smoke - Child Processes - Child Blocked On Direct Launch
	Given I run 'taskmgr.exe'
	When I click 'OK' on the elevation popup
	Then the process does not start
	And 1 local ECS event has been generated
	And the local ECS event 116 has the following details
		| file.path   |
		| taskmgr.exe |

#confirms exe's included in the client installer exist
@Smoke @EPM-17974
Scenario: Smoke - Installer - Check Bundled Applications Exist
	When the file 'C:\Program Files\Avecto\Privilege Guard Client\DefendpointService.exe' 'does' exist
	Then the file 'C:\Program Files\Avecto\Privilege Guard Client\PGSystemTray.exe' 'does' exist
	And the file 'C:\Program Files\Avecto\Privilege Guard Client\PGShellExecProxy.exe' 'does' exist
	And the file 'C:\Program Files\Avecto\Privilege Guard Client\PGMessageHostExt.exe' 'does' exist
	And the file 'C:\Program Files\Avecto\Privilege Guard Client\PGEmail.exe' 'does' exist
	And the file 'C:\Program Files\Avecto\Privilege Guard Client\PGCaptureConfig.exe' 'does' exist
	And the file 'C:\Program Files\Avecto\Privilege Guard Client\PGActiveXInstall.exe' 'does' exist