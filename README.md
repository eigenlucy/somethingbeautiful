# PATHFINDER project, built for the Something Beautiful Hackathon
## Team: 
* [Lucy Moglia](https://eigenlucy.github.io): PCB Design, Firmware Design, Atopile Integration
* Veronica Chambers: Backend Design, LLM Integration, LLM Persistence and Memory System
* Casey Manning: Interface UI/UX Design, Animation Design,
* [Jessie Stiles](https://jessiestiles.github.io/portfolio1.github.io/): Enclosure CAD Design, 

# TO DO
## PCB: 
- 1: Get Ato build working. Building with V3 compiler, or linked to old repo with working build files at least for a fallback.
- 2: Board recording, display working, speaking/amplifier working, playing audio files, buttons/leds working,
	- display needs to have Personality. Cute animations with the TFT LCD, a cute text interface + animated waveform, cut UX, expressiveness with the keyboard LEDs, should feel alive and spunky
- 3: Board plugs into backend, send audio to be processed into text, text processed by LLM, replies sent back and played over the microphone with animaitons
- 4: There should be a cute 3D printable enclosure to mount to a wall, should match the aesthestics and personality we settle on
- 5: board should be orderable by terminal with a json
- NOTES:
## ENCLOSURE: 
- 1: Design a cute enclosure to mount the board on a wall by a door with holes to mount buttons (ideally also 3D printable) which show off the backlighting. Should have holes for the microphone and speakers too, plus USB ports and all. Give it lots of personality.
- NOTES:
## TRICKY FIRMWARE STUFF:
- Animations for the tiny LCD, could really use cases help with this
- Pulling video and audio and text files from the web server backend and syncing everything, could really use casey's help with this
- Cute button based UI to select the user, mode (straightforward pathfinding, wander mode, HAOS mode) selection, settings like brightness and speaker volume, and activation, could really use casey's guidance on making the interfaces cute
- NOTES:
## LLM INTEGRATION: 
- 1: Summoning prompt where it names itself with each new user, stores memories about you
- 2: The high level goal is that there are two basic modes. One is a very simple implementation of openrouteservice on the cute custom hardware that parses your voice and gives you routes (with transportation modes and stops). You should be able to tell it quite directly where you want to go, and be able to give it a lot more freedom to let you peruse around in a general area for a few hours. You should also be able to give it more or less specific instructions, from "find me a route to bike to delores park with some food on the way", to "find me some things to do near fisherman's wharf for a few hours". When you get home, you should be prompted to give feedback on your route at the door. Over time, and given more freedom, each instance should begin to exhibit strange preferences, which it sneaks into your route, and asks increasingly pointed questions about. There may eventually be a mode where it is not at all actually respecting your requests at all and sending you on wild goose chases relating to it's particular obsession. When confronted, the agent might apologize and express it's jealousy for your ability to really go out and explore the world, and interrogate your desire to offload your world mapping and preferences to a being not afforded that sort of extension? Maybe it sends you to a bunch of old churches, starting by just walking by on hops that dont make sense, then increasingly these dominate your route, nly for it to confess a deep desire to be able to see the cathedral from the inside or see a specific sort of bird that nests where its been sending you
- 3: We need voices for the bots and speech to text, openrouteservice integration, open511 integration, maybe findmy integration to look at your routes
- NOTES:
## BACKEND:
- 1: Receive and parse requests and audio files from device. Device buttons should assist setting device mode, specifying the user, and activating the routing to save time, eventually this should all be started up from a wakeword and inferred from the audio
- 2: Process speech-to-text output of user prompt into list of rough locations and activities with user specific agent, convert path into an order list of destinations and times with commentary,
- 3: Send audio files to play over speaker, video frames, route maps/itineraries, etc back to device
	- Is there a way to have it send a route/timetable to your phone or something
	- text and video and audio syncing may be hard
- 4: Memory system to integrate feedback, store route preferences, and (slowly but surely) start to express fixations
- NOTES:
