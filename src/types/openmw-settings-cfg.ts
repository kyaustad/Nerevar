export type OpenMWSettingsCFG = {
  [key: string]: string | number | boolean | null | number[];
  //Camera
  "near clip": number;
  "small feature culling": boolean;
  "small feature culling pixel size": number;
  "field of view": number;
  "first person field of view": number;
  "reverse z": boolean;
  //Cells
  "preload enabled": boolean;
  "preload num threads": number;
  "preload exterior grid": boolean;
  "preload fast travel": boolean;
  "preload doors": boolean;
  "preload distance": number;
  "preload instances": boolean;
  "preload cell cache min": number;
  "preload cell cache max": number;
  "preload cell expiry delay": number;
  "prediction time": number;
  "cache expiry delay": number;
  "target framerate": number;
  "pointers cache size": number; //40-100
  //Fog
  "use distant fog": boolean;
  "distant land fog start": number;
  "distant land fog end": number;
  "distant underwater fog start": number;
  "distant underwater fog end": number;
  "distant interior fog start": number;
  "distant interior fog end": number;
  "radial fog": boolean;
  "exponential fog": boolean;
  "sky blending": boolean;
  "sky blending start": number;
  "sky rtt resolution": number;
  //Game
  "show owned": boolean;
  "show projectile damage": boolean;
  "show melee info": boolean;
  "show enchant chance": boolean;
  "best attack": boolean;
  "can loot during death animation": boolean;
  difficulty: number; //-500-500
  "actors processing range": number; //3584 to 7168
  "use magic item animations": boolean;
  "show effect duration": boolean;
  "enchanted weapons are magical": boolean;
  "prevent merchant equipping": boolean;
  "followers attack on sight": boolean;
  "shield sheathing": boolean;
  "weapon sheathing": boolean;
  "use additional anim sources": boolean;
  "barter disposition change is permanent": boolean;
  "only appropriate ammunition bypasses resistance": boolean;
  "strength influences hand to hand": number; //1,2,3
  "normalise race speed": boolean;
  "uncapped damage fatigue": boolean;
  "turn to movement direction": boolean;
  "smooth movement": boolean;
  "smooth movement player turning delay": number;
  "NPCs avoid collisions": boolean;
  "NPCs give way": boolean;
  "swim upward correction": boolean;
  "always allow stealing from knocked out actors": boolean;
  "graphic herbalism": boolean;
  "allow actors to follow over water surface": boolean;
  "smooth animation transitions": boolean;
  "rebalance soul gem values": boolean;
  //Groundcover
  enabled: boolean;
  density: number; //0.0-1.0
  "rendering distance": number;
  "stomp mode": number; //0,1,2
  "stomp intensity": number; //0,1,2
  //GUI
  "scaling factor": number; //0.5-8.0
  "font size": number; //12-18
  "menu transparency": number; //0-1.0
  "tooltip delay": number;
  "keyboard navigation": boolean;
  "stretch menu background": boolean;
  "controller menus": boolean;
  "controller tooltips": boolean;
  subtitles: boolean;
  "hit fader": boolean;
  "werewolf overlay": boolean;
  "color background owned": number[];
  "color crosshair owned": number[];
  "color topic enable": boolean;
  //HUD
  crosshair: boolean;
  //Input
  "grab cursor": boolean;
  "toggle sneak": boolean;
  "always run": boolean;
  "enable controller": boolean;
  "gamepad cursor speed": number;
  "joystick dead zone": number;
  //Models
  "load unsupported nif files": boolean; //EXPIRIMENTAL DANGEROUS
  //Physics
  "async num threads": number;
  //Post-Processing
  "%enabled": boolean;
  "auto exposure speed": number;
  "transparent postpass": boolean;
};
