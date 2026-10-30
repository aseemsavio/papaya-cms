#!/usr/bin/env python3
"""Seed the local Aperture demo: thumbnails, Markdown, and S3 uploads.

Run the local S3 endpoint first (see README), then:

    python3 scripts/seed_aperture.py

Source photographs live in .local-content/images/<pexels-id>.jpg and are
expected to have been downloaded at 1600px. Nothing is committed to git.
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

import boto3
from PIL import Image, ImageOps

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / ".local-content" / "images"
UPLOAD = ROOT / ".local-content" / "upload"
MARKDOWN = ROOT / ".local-content" / "markdown"

ENDPOINT = os.environ.get("S3_ENDPOINT", "http://127.0.0.1:9000")
BUCKET = os.environ.get("S3_BUCKET", "papaya")
ROOT_FOLDER = os.environ.get("S3_ROOT_FOLDER", "still").strip("/")
ACCESS_KEY = os.environ.get("S3_ACCESS_KEY", "test")
SECRET_KEY = os.environ.get("S3_SECRET_KEY", "testtest")
PUBLIC_BASE = os.environ.get("S3_PUBLIC_BASE", f"{ENDPOINT}/{BUCKET}")

KITS = {
    "a": {
        "camera": "Hasselblad X2D 100C",
        "lens": "XCD 45mm f/3.5",
        "aperture": "f/8",
        "shutter": "1/250",
        "iso": "64",
        "focal_length": "45mm",
        "white_balance": "Daylight · 5600K",
        "film": "Digital · medium format",
    },
    "b": {
        "camera": "Mamiya 7 II",
        "lens": "N 65mm f/4",
        "aperture": "f/11",
        "shutter": "1/125",
        "iso": "160",
        "focal_length": "65mm",
        "white_balance": "Daylight · 5500K",
        "film": "120 · Kodak Portra 160",
    },
    "c": {
        "camera": "Leica M11",
        "lens": "Summilux-M 50mm f/1.4",
        "aperture": "f/2",
        "shutter": "1/500",
        "iso": "200",
        "focal_length": "50mm",
        "white_balance": "Tungsten · 3200K",
        "film": "Digital · full frame",
    },
    "d": {
        "camera": "Fujifilm GFX 100S",
        "lens": "GF 32-64mm f/4",
        "aperture": "f/8",
        "shutter": "1/400",
        "iso": "100",
        "focal_length": "45mm",
        "white_balance": "Daylight · 5600K",
        "film": "Digital · medium format",
    },
    "e": {
        "camera": "Pentax 67",
        "lens": "SMC 105mm f/2.4",
        "aperture": "f/5.6",
        "shutter": "1/250",
        "iso": "400",
        "focal_length": "105mm",
        "white_balance": "Auto · 5200K",
        "film": "120 · Kodak Ektar 100",
    },
    "f": {
        "camera": "Sony α7R V",
        "lens": "FE 70-200mm f/2.8 GM II",
        "aperture": "f/5.6",
        "shutter": "1/640",
        "iso": "100",
        "focal_length": "135mm",
        "white_balance": "Daylight · 5600K",
        "film": "Digital · full frame",
    },
}

# Newest first. IDs map to .local-content/images/<id>.jpg.
POSTS = [
    {
        "id": "12527037",
        "slug": "still-water",
        "title": "Still Water",
        "place": "Lake Bled, Slovenia",
        "date": "2026-09-18T07:12:00Z",
        "tags": ["Landscape", "Water"],
        "kit": "a",
        "summary": "First light on a lake that refused to move.",
        "body": [
            "There was no wind at all for about four minutes. The far bank, the pines, the whole grey ceiling of cloud arrived in the water without a single seam, and the only evidence of depth was a faint gradient where the lake floor fell away.",
            "I waited for a ripple that never came, made three frames, and walked back to the car before the mist lifted. The silence is what I remember most — not the picture.",
        ],
    },
    {
        "id": "8869381",
        "slug": "ridge",
        "title": "Ridge",
        "place": "Sossusvlei, Namibia",
        "date": "2026-08-02T16:40:00Z",
        "tags": ["Landscape", "Minimal"],
        "kit": "b",
        "summary": "A single dune edge reduced to light and shadow.",
        "body": [
            "The dunes here are enormous, but from the right angle they compress into a single line. I stood on the shaded side and let the crest do all the work, cropping out the horizon so the frame became a study of one curve.",
            "Portra held the warmth in the sand without turning it orange, which is the whole reason I carried the Mamiya up that slope.",
        ],
    },
    {
        "id": "37186482",
        "slug": "sunrise-no-wind",
        "title": "Sunrise, No Wind",
        "place": "Loch Morlich, Scotland",
        "date": "2026-07-11T05:28:00Z",
        "tags": ["Landscape", "Water", "Dawn"],
        "kit": "a",
        "summary": "The Highlands, briefly borrowed from a postcard.",
        "body": [
            "I had planned for rain and got this instead: a soft pink wash over the hills and a loch flat enough to double the sky. The midges were ferocious, so the exposure had to be quick and decisive.",
            "Two frames, one step left, and the whole scene reorganised itself around the reflection. This is the second one.",
        ],
    },
    {
        "id": "8803309",
        "slug": "mist-and-stone",
        "title": "Mist and Stone",
        "place": "Lake Bohinj, Slovenia",
        "date": "2026-06-04T06:05:00Z",
        "tags": ["Landscape", "Minimal", "Fog"],
        "kit": "b",
        "summary": "One rock, one lake, and a horizon that dissolved.",
        "body": [
            "Fog does the tidying for you. By the time I found this stone the far shore had already disappeared, leaving a grey field with a single dark punctuation mark in it.",
            "I exposed for the mist and let the rock go nearly black. The negative has more detail than the print; I prefer it that way.",
        ],
    },
    {
        "id": "5392095",
        "slug": "reeds-at-dusk",
        "title": "Reeds at Dusk",
        "place": "Lake Hornborga, Sweden",
        "date": "2026-05-22T19:52:00Z",
        "tags": ["Landscape", "Water", "Dusk"],
        "kit": "a",
        "summary": "The last ten minutes of colour, held in the reeds.",
        "body": [
            "The sun had already gone behind the treeline, but the sky kept giving colour for another ten minutes. Reeds are perfect receivers for that kind of light — they hold it while everything else turns blue.",
            "I used a longish lens to compress the foreground into the water and keep the far bank as a thin dark band.",
        ],
    },
    {
        "id": "11866412",
        "slug": "low-fog-first-light",
        "title": "Low Fog, First Light",
        "place": "Richmond Park, London",
        "date": "2026-04-09T06:31:00Z",
        "tags": ["Landscape", "Fog", "Autumn"],
        "kit": "e",
        "summary": "A city park pretending it is the wilderness.",
        "body": [
            "Twenty minutes from a main road, the fog turned the park into something much older. The deer were somewhere off to the right; the water was doing all the talking.",
            "I like the way the bare branches break the reflection without destroying it. That tension is the photograph.",
        ],
    },
    {
        "id": "15526514",
        "slug": "reed-water",
        "title": "Reed Water",
        "place": "Lake Vättern, Sweden",
        "date": "2026-03-15T07:18:00Z",
        "tags": ["Landscape", "Water", "Minimal"],
        "kit": "a",
        "summary": "Teal water, dark reeds, no horizon line.",
        "body": [
            "I stood in the shallows with the tripod half sunk in mud and framed out the sky entirely. Without a horizon the water becomes a surface rather than a place, and the reeds read as brushstrokes.",
            "The cold made the shutter feel like a mechanism from another century. Worth it.",
        ],
    },
    {
        "id": "33828128",
        "slug": "pier-in-fog",
        "title": "Pier in Fog",
        "place": "Derwentwater, England",
        "date": "2026-02-18T08:04:00Z",
        "tags": ["Landscape", "Fog", "Minimal"],
        "kit": "b",
        "summary": "A jetty that goes nowhere, which is the point.",
        "body": [
            "The pier is a cliché in the right weather and a cliché in the wrong weather; the difference is whether you can see the far end. Here you cannot, so it becomes an instruction rather than an object.",
            "I placed it low and right, left the mist the subject, and stopped down to keep the near posts crisp.",
        ],
    },
    {
        "id": "18386434",
        "slug": "the-long-exposure",
        "title": "The Long Exposure",
        "place": "Skógafoss, Iceland",
        "date": "2026-01-27T14:22:00Z",
        "tags": ["Landscape", "Water", "Long Exposure"],
        "kit": "a",
        "summary": "Four seconds of falling water, flattened into silk.",
        "body": [
            "A ten-stop filter and four seconds turned the waterfall into something closer to smoke. The rock stays sharp because it is the only thing in the frame that is not moving.",
            "The hardest part was keeping spray off the front element. I shot six frames and this was the only clean one.",
        ],
    },
    {
        "id": "8340544",
        "slug": "fall",
        "title": "Fall",
        "place": "Seljalandsfoss, Iceland",
        "date": "2025-12-09T12:10:00Z",
        "tags": ["Landscape", "Water", "Vertical"],
        "kit": "e",
        "summary": "A tall frame for a tall fall.",
        "body": [
            "Some subjects tell you how they want to be framed. This one wanted the whole drop, from the lip of the cliff to the pool, with no compromise at either end.",
            "I metered for the water and let the rock go deep. The print is almost two metres tall and still feels compressed.",
        ],
    },
    {
        "id": "28357543",
        "slug": "green-ravine",
        "title": "Green Ravine",
        "place": "Khao Sok, Thailand",
        "date": "2025-11-21T10:45:00Z",
        "tags": ["Landscape", "Water", "Jungle"],
        "kit": "a",
        "summary": "A waterfall hidden inside the forest.",
        "body": [
            "The air was so humid that the lens fogged every time I stepped out from under the canopy. I waited it out, then made one frame before the light moved off the water.",
            "The palette is almost monochrome green, which makes the white water feel louder than it is.",
        ],
    },
    {
        "id": "34565944",
        "slug": "tracks-in-snow",
        "title": "Tracks in Snow",
        "place": "Hardangervidda, Norway",
        "date": "2025-10-30T13:05:00Z",
        "tags": ["Landscape", "Minimal", "Winter"],
        "kit": "b",
        "summary": "A line of footprints and nothing else for miles.",
        "body": [
            "The plateau was so featureless that the footprints became the landscape. I followed them for an hour before realising the photograph was behind me, looking back at the single line I had joined.",
            "No horizon, no scale, just a suggestion of direction. Sometimes that is enough.",
        ],
    },
    {
        "id": "11975905",
        "slug": "dune-line",
        "title": "Dune Line",
        "place": "Death Valley, California",
        "date": "2025-09-14T17:36:00Z",
        "tags": ["Landscape", "Minimal", "Desert"],
        "kit": "d",
        "summary": "The quietest thing in the desert.",
        "body": [
            "Late afternoon, no wind, and a dune ridge that read as a pencil line against the sky. I used a long lens to remove every distracting element and leave only the gradient of the sand.",
            "It is almost an abstract, but the faint texture at the bottom keeps it tethered to the real.",
        ],
    },
    {
        "id": "16908868",
        "slug": "erg-at-golden-hour",
        "title": "Erg at Golden Hour",
        "place": "Merzouga, Morocco",
        "date": "2025-08-23T18:12:00Z",
        "tags": ["Landscape", "Desert", "Colour"],
        "kit": "a",
        "summary": "Sand behaving like a fabric.",
        "body": [
            "The wind had combed the surface into fine parallel ridges. From the top of the dune they looked like corduroy, and the low sun turned every ridge into a tiny wall of shadow.",
            "I made this frame just before the light went from gold to orange, which is a window of about ninety seconds.",
        ],
    },
    {
        "id": "7271881",
        "slug": "red-dune",
        "title": "Red Dune",
        "place": "Sossusvlei, Namibia",
        "date": "2025-07-05T15:48:00Z",
        "tags": ["Landscape", "Desert", "Colour"],
        "kit": "b",
        "summary": "Iron oxide, blue sky, and a hard edge.",
        "body": [
            "The colour contrast here is almost too much: red sand against a saturated blue sky with nothing in between. I composed tight to keep the frame graphic rather than scenic.",
            "The dune's edge does the job a horizon usually does, separating two flat fields of colour.",
        ],
    },
    {
        "id": "25975000",
        "slug": "where-the-dune-meets-the-sea",
        "title": "Where the Dune Meets the Sea",
        "place": "Dune du Pilat, France",
        "date": "2025-06-12T11:20:00Z",
        "tags": ["Landscape", "Coast", "Aerial"],
        "kit": "d",
        "summary": "Sand and surf, divided by a single curve.",
        "body": [
            "From the top of the dune the coastline becomes an abstraction: a field of ochre, a field of blue, and a white seam where they meet.",
            "I waited for the foam to draw a clean line before pressing the shutter. The wave did most of the composition.",
        ],
    },
    {
        "id": "35531268",
        "slug": "beach-grass",
        "title": "Beach Grass",
        "place": "Camargue, France",
        "date": "2025-05-02T09:16:00Z",
        "tags": ["Landscape", "Coast", "Minimal"],
        "kit": "a",
        "summary": "Dune grass against a washed-out sky.",
        "body": [
            "The wind was strong enough to keep the grass in constant motion, so I picked a fast shutter and let the blur happen in the background instead.",
            "White sand, pale sky, a few dark strokes of vegetation. The whole picture is basically three values.",
        ],
    },
    {
        "id": "15124461",
        "slug": "headland-late-afternoon",
        "title": "Headland, Late Afternoon",
        "place": "Big Sur, California",
        "date": "2025-03-26T16:55:00Z",
        "tags": ["Landscape", "Coast"],
        "kit": "d",
        "summary": "The Pacific doing its slow collapse against the cliffs.",
        "body": [
            "The coast here is vertical in a way that photographs struggle with. I used the headland as a diagonal and let the ocean fill the rest, keeping the horizon high so the land has weight.",
            "The haze did the atmospheric perspective for me; no filter required.",
        ],
    },
    {
        "id": "3533909",
        "slug": "open-water",
        "title": "Open Water",
        "place": "Bonifacio, Corsica",
        "date": "2025-02-17T12:42:00Z",
        "tags": ["Landscape", "Coast", "Water"],
        "kit": "a",
        "summary": "Turquoise, white limestone, and a lot of wind.",
        "body": [
            "The sea was too rough for a calm reflection, so I leaned into the texture instead. The cliffs gave me a stable anchor on the left and the water did the rest.",
            "This is the kind of colour that looks fake on a screen and completely natural when you are standing in it.",
        ],
    },
    {
        "id": "33992262",
        "slug": "cliff-edge",
        "title": "Cliff Edge",
        "place": "Étretat, France",
        "date": "2025-01-08T10:24:00Z",
        "tags": ["Landscape", "Coast", "Vertical"],
        "kit": "b",
        "summary": "Chalk, sea, and a very long way down.",
        "body": [
            "I wanted the cliff to feel as tall as it is, so I shot vertically and put the horizon near the top. The path on the edge gives the scale a human measure.",
            "The light was flat, which is usually a problem, but flat light is kind to white rock.",
        ],
    },
    {
        "id": "18562226",
        "slug": "salt-and-stone",
        "title": "Salt and Stone",
        "place": "Costa Vicentina, Portugal",
        "date": "2024-12-19T14:08:00Z",
        "tags": ["Landscape", "Coast", "Minimal"],
        "kit": "a",
        "summary": "A pale shore under a very blue sky.",
        "body": [
            "The rock here is almost white and the water almost unreal. I exposed for the highlights and let the shadows settle into a soft grey, which kept the whole frame airy.",
            "There is very little in the picture, and that is what I like about it.",
        ],
    },
    {
        "id": "5328309",
        "slug": "green-cliff",
        "title": "Green Cliff",
        "place": "Faroe Islands",
        "date": "2024-11-03T11:37:00Z",
        "tags": ["Landscape", "Coast", "Green"],
        "kit": "e",
        "summary": "Grass, basalt, and weather arriving from the left.",
        "body": [
            "The Faroes make you work for a clean frame: the weather changes every few minutes and the wind never stops. This was a brief gap between squalls.",
            "The green is intense but accurate. Ektar loves that kind of light.",
        ],
    },
    {
        "id": "27786033",
        "slug": "mere",
        "title": "Mere",
        "place": "Lake District, England",
        "date": "2024-10-15T07:50:00Z",
        "tags": ["Landscape", "Water", "Minimal"],
        "kit": "b",
        "summary": "A lake with the colour turned almost all the way down.",
        "body": [
            "A grey morning, a grey lake, and a thin dark band of shore. I framed out the sky to keep the image quiet and let the water's texture carry the detail.",
            "It is the least dramatic photograph in the archive and one of my favourites.",
        ],
    },
    {
        "id": "12190785",
        "slug": "before-the-thaw",
        "title": "Before the Thaw",
        "place": "Lapland, Finland",
        "date": "2024-09-06T06:22:00Z",
        "tags": ["Landscape", "Fog", "Cold"],
        "kit": "a",
        "summary": "Bare trees standing in a flooded marsh.",
        "body": [
            "The marsh floods every spring and the trees spend a few weeks standing in still water. The effect is a forest with a second, softer forest underneath it.",
            "Blue hour, no wind, and a very long walk in the dark to get there.",
        ],
    },
    {
        "id": "3013956",
        "slug": "chalk-country",
        "title": "Chalk Country",
        "place": "Seven Sisters, England",
        "date": "2024-08-11T15:14:00Z",
        "tags": ["Landscape", "Coast", "Green"],
        "kit": "d",
        "summary": "White cliffs and green hills in horizontal layers.",
        "body": [
            "The landscape here is built out of bands: grass, chalk, shingle, sea. I found a high point and let the layers stack up, using the path as a leading line.",
            "The clouds were moving fast, so I made the frame between two shadows crossing the hills.",
        ],
    },
    {
        "id": "20029555",
        "slug": "mira-window-light",
        "title": "Mira, Window Light",
        "place": "Lisbon, Portugal",
        "date": "2024-07-02T17:26:00Z",
        "tags": ["Portrait", "Natural Light"],
        "kit": "c",
        "summary": "A portrait made with one window and no hurry.",
        "body": [
            "Mira sat by the window for an hour while the light moved across the room. We talked more than we shot. This frame came near the end, when the light had gone soft and she had stopped posing.",
            "The 50mm at f/2 keeps her sharp and lets the room fall away without disappearing entirely.",
        ],
    },
    {
        "id": "18925606",
        "slug": "half-portrait",
        "title": "Half Portrait",
        "place": "Berlin, Germany",
        "date": "2024-06-08T13:02:00Z",
        "tags": ["Portrait", "Close-up"],
        "kit": "c",
        "summary": "A face cropped to the essentials.",
        "body": [
            "I crop portraits aggressively when the eyes are doing enough work on their own. Here the frame cuts away the context and leaves only attention.",
            "The skin tones are almost unedited. Window light, a white wall, and a subject who knows how to be still.",
        ],
    },
    {
        "id": "32673412",
        "slug": "white-room",
        "title": "White Room",
        "place": "Copenhagen, Denmark",
        "date": "2024-05-14T11:48:00Z",
        "tags": ["Portrait", "High Key"],
        "kit": "c",
        "summary": "A high-key portrait with almost no shadow.",
        "body": [
            "The room was white, the dress was white, and the light was white. Without shadow the body becomes a shape, which is a different kind of portrait than the usual.",
            "I metered for the wall and let the highlights sit just below clipping.",
        ],
    },
    {
        "id": "31987755",
        "slug": "sara-north-window",
        "title": "Sara, North Window",
        "place": "Oslo, Norway",
        "date": "2024-04-03T10:15:00Z",
        "tags": ["Portrait", "Interior"],
        "kit": "c",
        "summary": "North light, warm wood, and a direct gaze.",
        "body": [
            "North light is the most forgiving light there is: soft, cool, and constant. Sara sat against the window frame and the room did the rest.",
            "I made two frames in this session. This is the one where she is looking at the camera.",
        ],
    },
    {
        "id": "31987756",
        "slug": "sara-second-frame",
        "title": "Sara, Second Frame",
        "place": "Oslo, Norway",
        "date": "2024-04-03T10:18:00Z",
        "tags": ["Portrait", "Interior"],
        "kit": "c",
        "summary": "The same light, a different attention.",
        "body": [
            "Three minutes after the previous frame, she turned slightly and the whole photograph changed. Portrait sessions are mostly waiting for a person to forget the camera is there.",
            "I kept both because they are not variations; they are two different sentences.",
        ],
    },
    {
        "id": "38540875",
        "slug": "amber",
        "title": "Amber",
        "place": "Marrakech, Morocco",
        "date": "2024-03-10T16:34:00Z",
        "tags": ["Portrait", "Colour"],
        "kit": "c",
        "summary": "A portrait that behaves like a warm room.",
        "body": [
            "The light in Marrakech at the end of the day is almost physical. I let it come through a doorway and fall across her face, keeping the background dark so the colour has somewhere to sit.",
            "There is very little retouching here beyond dust removal. The warmth is all in the hour.",
        ],
    },
    {
        "id": "32602425",
        "slug": "wing-wave",
        "title": "Wing / Wave",
        "place": "Pacific Grove, California",
        "date": "2024-02-01T08:26:00Z",
        "tags": ["Abstract", "Black & White", "Coast"],
        "kit": "f",
        "summary": "A wave that looks like a wing, in black and white.",
        "body": [
            "The wave curled at exactly the moment a bird crossed the frame, and for a second the two shapes rhymed. I did not see the bird until I looked at the contact sheet.",
            "Converted with a red filter in mind, printed high-contrast. The white is the subject.",
        ],
    },
    {
        "id": "18927540",
        "slug": "shaft",
        "title": "Shaft",
        "place": "Kyoto, Japan",
        "date": "2024-01-12T14:12:00Z",
        "tags": ["Architecture", "Light"],
        "kit": "d",
        "summary": "A concrete room with one decisive beam of light.",
        "body": [
            "The building was mostly shadow, but one shaft of afternoon light cut diagonally across the wall and floor. I waited until it reached the corner and then made the frame.",
            "Architecture photography is often about patience with the sun. Ten minutes later the picture was gone.",
        ],
    },
    {
        "id": "30699852",
        "slug": "empty-room-4pm",
        "title": "Empty Room, 4pm",
        "place": "Lisbon, Portugal",
        "date": "2023-12-02T16:02:00Z",
        "tags": ["Architecture", "Interior", "Minimal"],
        "kit": "d",
        "summary": "An empty room doing everything a subject needs to do.",
        "body": [
            "No furniture, no people, just a rectangle of light on the floor and a doorway to another rectangle. I stood in the corner and let the geometry line up.",
            "The room is not really empty; it is full of the shape of the afternoon.",
        ],
    },
    {
        "id": "38344729",
        "slug": "passage",
        "title": "Passage",
        "place": "Copenhagen, Denmark",
        "date": "2023-11-08T09:40:00Z",
        "tags": ["Architecture", "Interior", "Light"],
        "kit": "d",
        "summary": "A corridor as a study in perspective.",
        "body": [
            "The corridor had one window at the end and nothing else, which made it a perfect camera obscura of a space. Every line pointed to the light.",
            "I placed the camera dead centre and resisted the urge to tilt. The slight asymmetry of the doorways keeps it from feeling like a diagram.",
        ],
    },
    {
        "id": "4466711",
        "slug": "atrium-after-hours",
        "title": "Atrium, After Hours",
        "place": "Hamburg, Germany",
        "date": "2023-10-17T21:18:00Z",
        "tags": ["Architecture", "Night", "Colour"],
        "kit": "f",
        "summary": "A dark atrium with a single warm structure.",
        "body": [
            "The building was closed, but the atrium lights stayed on. The result was a black frame with one luminous object floating in it, like a diagram of the building's bones.",
            "I exposed for the highlights and let the shadows go completely black. Sometimes information is the enemy of atmosphere.",
        ],
    },
    {
        "id": "5015876",
        "slug": "shadow-study",
        "title": "Shadow Study",
        "place": "Barcelona, Spain",
        "date": "2023-09-05T15:26:00Z",
        "tags": ["Abstract", "Shadow", "Minimal"],
        "kit": "f",
        "summary": "Light on a wall, arranged by architecture.",
        "body": [
            "This is not a picture of a wall. It is a picture of the shape the wall made out of the sun. I found it in a stairwell and spent ten minutes waiting for the edge to soften.",
            "The frame is almost entirely empty, which is its own kind of composition.",
        ],
    },
    {
        "id": "15779815",
        "slug": "grid-study-no-4",
        "title": "Grid Study No. 4",
        "place": "Berlin, Germany",
        "date": "2023-08-09T12:08:00Z",
        "tags": ["Abstract", "Architecture", "Pattern"],
        "kit": "d",
        "summary": "A facade reduced to rhythm.",
        "body": [
            "The concrete grid repeats until it stops being a building and becomes a texture. I used a long lens to flatten the perspective and keep every cell the same size.",
            "The small irregularities — a dark window, a stain, a missing panel — are what keep the pattern alive.",
        ],
    },
    {
        "id": "34868966",
        "slug": "concrete-rhythm",
        "title": "Concrete Rhythm",
        "place": "Marseille, France",
        "date": "2023-07-15T10:32:00Z",
        "tags": ["Abstract", "Architecture", "Pattern"],
        "kit": "d",
        "summary": "Blocks of concrete catching the same light differently.",
        "body": [
            "Every block faced a slightly different direction, so the same sun produced a dozen different greys. The facade became a score.",
            "I shot it straight on, with the camera level, and let the repetition do the work.",
        ],
    },
    {
        "id": "9859862",
        "slug": "white-geometry",
        "title": "White Geometry",
        "place": "Los Angeles, California",
        "date": "2023-06-11T13:55:00Z",
        "tags": ["Architecture", "Minimal", "White"],
        "kit": "d",
        "summary": "A white building against a white sky.",
        "body": [
            "The sky was blown out and the building was white, so the photograph became a conversation between two nearly identical tones. Only the edges and shadows describe the form.",
            "I cropped tight to remove the street and the trees. The building floats.",
        ],
    },
    {
        "id": "37097374",
        "slug": "weave",
        "title": "Weave",
        "place": "Oaxaca, Mexico",
        "date": "2023-05-06T11:44:00Z",
        "tags": ["Abstract", "Texture", "Pattern"],
        "kit": "f",
        "summary": "A textile read as a landscape.",
        "body": [
            "I photographed this cloth on a loom with a long lens and no context. At this scale the weave becomes topography: ridges, valleys, a horizon of thread.",
            "The darks are almost black and the lights almost silver. It is a photograph of structure rather than of an object.",
        ],
    },
    {
        "id": "19669664",
        "slug": "cross-shadow",
        "title": "Cross Shadow",
        "place": "Seville, Spain",
        "date": "2023-04-02T18:06:00Z",
        "tags": ["Abstract", "Shadow", "Architecture"],
        "kit": "f",
        "summary": "A window frame casting a cross on a warm wall.",
        "body": [
            "Late afternoon in Seville turns every wall into a screen. This cross appeared for about twenty minutes and then climbed out of the frame.",
            "I exposed for the wall's warmth and let the shadow stay deliberately soft at the edges.",
        ],
    },
]


def average_color(path: Path) -> str:
    with Image.open(path) as im:
        im = ImageOps.exif_transpose(im).convert("RGB")
        small = im.resize((24, 24))
        pixels = list(small.getdata())
    count = len(pixels)
    r = sum(p[0] for p in pixels) // count
    g = sum(p[1] for p in pixels) // count
    b = sum(p[2] for p in pixels) // count
    return "#{:02x}{:02x}{:02x}".format(r, g, b)


def make_thumb(path: Path, destination: Path) -> tuple[int, int]:
    with Image.open(path) as im:
        im = ImageOps.exif_transpose(im).convert("RGB")
        width, height = im.size
        target = 1100
        if width > target:
            im = im.resize((target, round(height * target / width)), Image.LANCZOS)
        destination.parent.mkdir(parents=True, exist_ok=True)
        im.save(destination, "JPEG", quality=82, optimize=True, progressive=True)
        return width, height


def yaml_string(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def render_markdown(post: dict, width: int, height: int, color: str) -> str:
    kit = KITS[post["kit"]]
    aspect = round(width / height, 4)
    image_url = f"{PUBLIC_BASE}/{ROOT_FOLDER}/assets/images/{post['slug']}.jpg"
    thumb_url = f"{PUBLIC_BASE}/{ROOT_FOLDER}/assets/images/{post['slug']}-thumb.jpg"

    lines = [
        "---",
        f"title: {yaml_string(post['title'])}",
        f"slug: {yaml_string(post['slug'])}",
        f"date: {yaml_string(post['date'])}",
        f"summary: {yaml_string(post['summary'])}",
        f"author: {yaml_string('Aseem Savio')}",
        f"cover_image: {yaml_string(image_url)}",
        "tags:",
    ]
    for tag in post["tags"]:
        lines.append(f"  - {yaml_string(tag)}")
    lines += [
        "photo:",
        f"  camera: {yaml_string(kit['camera'])}",
        f"  lens: {yaml_string(kit['lens'])}",
        f"  aperture: {yaml_string(kit['aperture'])}",
        f"  shutter: {yaml_string(kit['shutter'])}",
        f"  iso: {yaml_string(kit['iso'])}",
        f"  focal_length: {yaml_string(kit['focal_length'])}",
        f"  white_balance: {yaml_string(kit['white_balance'])}",
        f"  film: {yaml_string(kit['film'])}",
        f"  location: {yaml_string(post['place'])}",
        f"  aspect: {aspect}",
        f"  thumb: {yaml_string(thumb_url)}",
        f"  color: {yaml_string(color)}",
        "---",
        "",
    ]
    for paragraph in post["body"]:
        lines.append(paragraph)
        lines.append("")
    return "\n".join(lines)


def main() -> int:
    missing = [p["id"] for p in POSTS if not (SOURCE / f"{p['id']}.jpg").exists()]
    if missing:
        print("Missing source images:", ", ".join(missing), file=sys.stderr)
        return 1

    if len({p["slug"] for p in POSTS}) != len(POSTS):
        print("Duplicate slugs in POSTS", file=sys.stderr)
        return 1

    s3 = boto3.client(
        "s3",
        endpoint_url=ENDPOINT,
        aws_access_key_id=ACCESS_KEY,
        aws_secret_access_key=SECRET_KEY,
        region_name="us-east-1",
    )

    try:
        s3.head_bucket(Bucket=BUCKET)
    except Exception:
        s3.create_bucket(Bucket=BUCKET)

    # Fresh demo: drop anything left from an older seed, including content
    # uploaded before the site had a root folder.
    stale = []
    for prefix in (f"{ROOT_FOLDER}/", "markdown/", "assets/"):
        listing = s3.list_objects_v2(Bucket=BUCKET, Prefix=prefix).get("Contents", [])
        stale += [{"Key": obj["Key"]} for obj in listing]
    if stale:
        s3.delete_objects(Bucket=BUCKET, Delete={"Objects": stale})

    MARKDOWN.mkdir(parents=True, exist_ok=True)
    (UPLOAD / "assets" / "images").mkdir(parents=True, exist_ok=True)

    for index, post in enumerate(POSTS, start=1):
        source = SOURCE / f"{post['id']}.jpg"
        slug = post["slug"]
        thumb_path = UPLOAD / "assets" / "images" / f"{slug}-thumb.jpg"

        width, height = make_thumb(source, thumb_path)
        color = average_color(source)
        markdown = render_markdown(post, width, height, color)

        (MARKDOWN / f"{slug}.md").write_text(markdown, encoding="utf-8")

        s3.put_object(
            Bucket=BUCKET,
            Key=f"{ROOT_FOLDER}/assets/images/{slug}.jpg",
            Body=source.read_bytes(),
            ContentType="image/jpeg",
            CacheControl="public, max-age=31536000, immutable",
        )
        s3.put_object(
            Bucket=BUCKET,
            Key=f"{ROOT_FOLDER}/assets/images/{slug}-thumb.jpg",
            Body=thumb_path.read_bytes(),
            ContentType="image/jpeg",
            CacheControl="public, max-age=31536000, immutable",
        )
        s3.put_object(
            Bucket=BUCKET,
            Key=f"{ROOT_FOLDER}/markdown/{slug}.md",
            Body=markdown.encode("utf-8"),
            ContentType="text/markdown; charset=utf-8",
            CacheControl="public, max-age=60",
        )

        print(f"[{index:02d}/{len(POSTS)}] {slug}  {width}x{height}  {color}")

    policy = {
        "Version": "2012-10-17",
        "Statement": [
            {
                "Effect": "Allow",
                "Principal": "*",
                "Action": ["s3:GetObject"],
                "Resource": [f"arn:aws:s3:::{BUCKET}/{ROOT_FOLDER}/assets/*"],
            }
        ],
    }
    s3.put_bucket_policy(Bucket=BUCKET, Policy=json.dumps(policy))

    print(f"\nSeeded {len(POSTS)} posts into {ENDPOINT}/{BUCKET}/{ROOT_FOLDER}")
    print(f"Markdown: {MARKDOWN}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
