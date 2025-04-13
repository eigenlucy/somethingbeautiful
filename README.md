# PATHFINDER project, built for the Something Beautiful Hackathon
## Team: 
* [Lucy Moglia](https://eigenlucy.github.io): PCB Design, Firmware Design, Atopile Integration
* [Veronica Chambers](https://www.linkedin.com/in/victoria-cabrera-moglia/): Backend Design, LLM Integration, LLM Persistence and Memory System
* [Jessie Stiles](https://jessiestiles.github.io/portfolio1.github.io/): Enclosure CAD Design,
* [Yoyo](https://exanova-y.github.io): UX Design, pathfinding, route visualization

## High level overview:
ESP32-S3 microcontroller,  a MAX98357 I2S digital amplifier (esp-hal i2s audio library), a ST7735S based 128 * 160 pixel LCD (st7735-lcd crate), and  WS2812B Neopixels (smart_leds crate)

Route and activity planning bot that lives in a panel beside your door. Tell it where you want to go, how much of a rush you're in, what kind of activities you are looking for wtc, and it will troll through a variety of databases to plan your route. Much is left to the personality of the agent associated with the individual user. 

## Notes:
* Using commit df3bad4 for PCB
* We should all settle on some example prompts, eg "I'd like to spend two hours getting lunch by the pier"

# References/Dependencies:
* [OpenRouteService](https://openrouteservice.org/)
* [OpenRouteService Route Visualization Guide](https://medium.com/@atulvpoddar4/visualizing-routes-with-real-data-a-python-guide-to-interactive-mapping-db14189cf185)
* [Izzymonitor Project Page](https://eigenlucy.github.io/projects/izzymonitor/) and [Repo with pin refs](https://github.com/eigenlucy/ESPHome-Panel/tree/izzymonitor/)
* [Izzymonitor actions run associated with PCB on hand](https://github.com/eigenlucy/ESPHome-Panel/actions/runs/13046416119). Build artifacts in Actions run contain Gerbers+PCBA files.

# Open Questions:
* Do we want to use python, rust, or cpp for esp32s3. We have some [Rust firmware working](https://github.com/izzyhub/izzymonitor-firmware) to a limited degree. Micropython is a solid option too, and might make OTAA updates easier.
* How much of a pain is it going to be to give agents this much freedom to troll through databases while actually being interesting?

# TO DO
## PCB/Basic Firmware: 
- 1: Get Ato build working. Building with V3 compiler, or linked to old repo with working build files at least for a fallback.
- 2: Board recording, display working, speaking/amplifier working, playing audio files, buttons/leds working,
- 3: board should be able to be ordered via terminal with a json config with address and payment info
- NOTES:
## ENCLOSURE: 
- 1: Design a cute enclosure to mount the board on a wall by a door with holes to mount buttons (ideally also 3D printable) which show off the backlighting. Should have holes for the microphone and speakers too, plus USB ports and all. Give it lots of personality.
- NOTES:
## FIRMWARE:
- Animations for the tiny LCD, could really use cases help with this
- display needs to have Personality. Cute animations with the TFT LCD, a cute text interface + animated waveform, cut UX, expressiveness with the keyboard LEDs, should feel alive and spunky
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

