<CsoundSynthesizer>
<CsOptions>
-odac
</CsOptions>

<CsInstruments>

sr = 44100
ksmps = 64
nchnls = 2
0dbfs  = 1

chn_k "gain", 1, 2, 0.6, 0, 1
chnset 0.6, "gain"

instr 1

kamp = .6
kcps = 220

kgain chnget "gain"
asig oscil (kgain * kamp), kcps
     out asig, asig

endin

instr 2

kgain chnget "gain"
chnset kgain, "showGain" 

endin

</CsInstruments>
<CsScore>

f0 100000
i 1  0 4 1
i 2  0 -1
e
</CsScore>
</CsoundSynthesizer>

<Carrot>
config:
  size:
    width: 400
    height: 300
state:
  init:
    gain: 0.5
    showGain: 0.0
ui:
  hor:
    - knob: gain
    - knob: showGain
csound:
  read: [showGain]
</Carrot>
