param(
    [string]$IconPath = "D:\Projects\bitg\docs\brand\bitg-icon-concept-v1.png",
    [string]$LogoPath = "D:\Projects\bitg\docs\brand\bitg-logo-concept-v1.png",
    [string]$OutputPath = "D:\Projects\bitg\docs\brand\motion\bitg-teaser-v1.mp4",
    [switch]$Force
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
    throw "ffmpeg is required but was not found in PATH."
}

foreach ($asset in @($IconPath, $LogoPath)) {
    if (-not (Test-Path -LiteralPath $asset)) {
        throw "Missing source asset: $asset"
    }
}

if ((Test-Path -LiteralPath $OutputPath) -and -not $Force) {
    throw "Output already exists: $OutputPath. Pass -Force to replace it."
}

$outputDir = Split-Path -Parent $OutputPath
$buildDir = "D:\Projects\bitg\target\brand-teaser-v1"
New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
New-Item -ItemType Directory -Path $buildDir -Force | Out-Null

$iconOutline = Join-Path $buildDir "icon-outline.png"
$logoOutline = Join-Path $buildDir "logo-outline.png"
$logoWhite = Join-Path $buildDir "logo-white.png"

function Invoke-Ffmpeg {
    param([string[]]$Arguments)

    & ffmpeg -hide_banner -loglevel warning @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "ffmpeg failed with exit code $LASTEXITCODE"
    }
}

# Derive motion-friendly outline and monochrome assets from the transparent masters.
Invoke-Ffmpeg @(
    "-y", "-i", $IconPath,
    "-filter_complex", "color=c=0x39E8FF:s=1254x1254,format=rgba[c];[0:v]alphaextract,edgedetect=low=0.04:high=0.18,dilation[m];[c][m]alphamerge[out]",
    "-map", "[out]", "-frames:v", "1", $iconOutline
)

Invoke-Ffmpeg @(
    "-y", "-i", $LogoPath,
    "-filter_complex", "color=c=0x34DFF2:s=2172x724,format=rgba[c];[0:v]alphaextract,edgedetect=low=0.04:high=0.18,dilation[m];[c][m]alphamerge[out]",
    "-map", "[out]", "-frames:v", "1", $logoOutline
)

Invoke-Ffmpeg @(
    "-y", "-i", $LogoPath,
    "-filter_complex", "color=c=white:s=2172x724,format=rgba[c];[0:v]alphaextract[m];[c][m]alphamerge[out]",
    "-map", "[out]", "-frames:v", "1", $logoWhite
)

$sceneDuration = "3.2"
$renderArgs = @("-c:v", "libx264", "-preset", "medium", "-crf", "18", "-pix_fmt", "yuv420p", "-r", "30", "-an")

$scene1 = Join-Path $buildDir "scene-01.mp4"
$scene1Filter = @'
[0:v]noise=alls=3:allf=t+u,vignette=PI/5[bg];
[1:v]format=rgba,scale=w='480+90*min(t/2.4,1)':h=-1:eval=frame,rotate='(-3+3*min(t/2.2,1))*PI/180':ow=rotw(iw):oh=roth(ih):c=none,fade=t=in:st=0.10:d=0.75:alpha=1,fade=t=out:st=2.75:d=0.35:alpha=1[mark];
[bg][mark]overlay=x='(W-w)/2':y='(H-h)/2+24*sin(t*2.1)':format=auto,
drawbox=x=150:y='260+430*t':w=780:h=3:color=0x48E7F2@0.55:t=fill:enable='between(t,0.2,2.8)',
drawtext=fontfile='C\:/Windows/Fonts/seguisb.ttf':text='BITG / FORM 01':fontcolor=0x6FE7EF@0.85:fontsize=27:x=76:y=110,
drawtext=fontfile='C\:/Windows/Fonts/segoeui.ttf':text='NEW ALPHABET SYSTEM':fontcolor=white@0.42:fontsize=20:x=76:y=151[out]
'@
$scene1Args = @(
    "-y", "-f", "lavfi", "-i", "gradients=s=1080x1920:r=30:c0=0x01040B:c1=0x071A37:c2=0x003B5E:n=3:type=radial:speed=0.035:d=$sceneDuration",
    "-loop", "1", "-framerate", "30", "-i", $iconOutline,
    "-filter_complex", $scene1Filter, "-map", "[out]", "-t", $sceneDuration
) + $renderArgs + @($scene1)
Invoke-Ffmpeg $scene1Args

$scene2 = Join-Path $buildDir "scene-02.mp4"
$scene2Filter = @'
[0:v]noise=alls=2:allf=t+u,vignette=PI/5[bg];
[1:v]format=rgba,scale=660:660,split=5[fullbase][shadowbase][topbase][midbase][botbase];
[topbase]crop=660:220:0:0[top];
[midbase]crop=660:220:0:220[mid];
[botbase]crop=660:220:0:440[bot];
[shadowbase]colorchannelmixer=rr=0.05:gg=0.16:bb=0.34:aa=0.70,gblur=sigma=9[shadow];
[fullbase]fade=t=in:st=0.92:d=0.30:alpha=1[full];
[bg][shadow]overlay=x='(W-w)/2+18':y='(H-h)/2+38':format=auto:enable='gte(t,0.88)'[s0];
[s0][top]overlay=x='(W-w)/2-700*max(0,1-t/0.95)':y=630:format=auto:enable='lte(t,1.18)'[s1];
[s1][mid]overlay=x='(W-w)/2+700*max(0,1-t/0.95)':y=850:format=auto:enable='lte(t,1.18)'[s2];
[s2][bot]overlay=x='(W-w)/2-700*max(0,1-t/0.95)':y=1070:format=auto:enable='lte(t,1.18)'[s3];
[s3][full]overlay=x='(W-w)/2':y='(H-h)/2-12+10*sin(t*2.8)':format=auto,
drawtext=fontfile='C\:/Windows/Fonts/seguisb.ttf':text='BUILDING THE GLYPH':fontcolor=white@0.58:fontsize=23:x=(w-text_w)/2:y=1420,
drawbox=x='210+660*min(t/2.6,1)':y=1471:w=170:h=3:color=0x2BE0E7@0.75:t=fill[out]
'@
$scene2Args = @(
    "-y", "-f", "lavfi", "-i", "gradients=s=1080x1920:r=30:c0=0x020611:c1=0x082354:c2=0x006D83:n=3:type=radial:speed=0.045:d=$sceneDuration",
    "-loop", "1", "-framerate", "30", "-i", $IconPath,
    "-filter_complex", $scene2Filter, "-map", "[out]", "-t", $sceneDuration
) + $renderArgs + @($scene2)
Invoke-Ffmpeg $scene2Args

$scene3 = Join-Path $buildDir "scene-03.mp4"
$scene3Filter = @'
[0:v]noise=alls=1:allf=t+u[bg];
[1:v]format=rgba,scale=900:-1,fade=t=in:st=0.05:d=0.55:alpha=1[outline];
[2:v]format=rgba,scale=w='830+35*min(t/2.4,1)':h=-1:eval=frame,fade=t=in:st=1.05:d=0.62:alpha=1[logo];
[bg][outline]overlay=x='(W-w)/2-85+85*min(t/1.2,1)':y='(H-h)/2-24':format=auto[s1];
[s1][logo]overlay=x='(W-w)/2':y='(H-h)/2':format=auto,
drawbox=x='-220+620*t':y=420:w=150:h=1080:color=0x22DCE9@0.08:t=fill:enable='between(t,0.1,2.2)',
drawtext=fontfile='C\:/Windows/Fonts/seguisb.ttf':text='IDENTITY / WORDMARK':fontcolor=0x0A1C46@0.46:fontsize=23:x=76:y=110[out]
'@
$scene3Args = @(
    "-y", "-f", "lavfi", "-i", "gradients=s=1080x1920:r=30:c0=0xF8FBFF:c1=0xDCEBFA:c2=0x9FEAF2:n=3:type=radial:speed=0.025:d=$sceneDuration",
    "-loop", "1", "-framerate", "30", "-i", $logoOutline,
    "-loop", "1", "-framerate", "30", "-i", $LogoPath,
    "-filter_complex", $scene3Filter, "-map", "[out]", "-t", $sceneDuration
) + $renderArgs + @($scene3)
Invoke-Ffmpeg $scene3Args

$scene4 = Join-Path $buildDir "scene-04.mp4"
$scene4Filter = @'
[0:v]noise=alls=3:allf=t+u,vignette=PI/5[bg];
[1:v]format=rgba,scale=w='1040-180*min(t/2.5,1)':h=-1:eval=frame,rotate='(1.4-1.4*min(t/2.2,1))*PI/180':ow=rotw(iw):oh=roth(ih):c=none,split=2[front][depth];
[depth]colorchannelmixer=rr=0.00:gg=0.15:bb=0.35:aa=0.72,gblur=sigma=3[shadow];
[bg][shadow]overlay=x='(W-w)/2+28':y='(H-h)/2+42':format=auto[s1];
[s1][front]overlay=x='(W-w)/2':y='(H-h)/2':format=auto,
drawbox=x='-180+500*t':y=0:w=110:h=1920:color=white@0.08:t=fill,
drawtext=fontfile='C\:/Windows/Fonts/segoeui.ttf':text='ONE SYSTEM. ONE MARK.':fontcolor=white@0.62:fontsize=23:x=(w-text_w)/2:y=1260[out]
'@
$scene4Args = @(
    "-y", "-f", "lavfi", "-i", "gradients=s=1080x1920:r=30:c0=0x06132F:c1=0x005BD1:c2=0x00C4D8:n=3:type=radial:speed=0.06:d=$sceneDuration",
    "-loop", "1", "-framerate", "30", "-i", $logoWhite,
    "-filter_complex", $scene4Filter, "-map", "[out]", "-t", $sceneDuration
) + $renderArgs + @($scene4)
Invoke-Ffmpeg $scene4Args

$scene5 = Join-Path $buildDir "scene-05.mp4"
$scene5Filter = @'
[0:v]noise=alls=1:allf=t+u[bg];
[1:v]format=rgba,scale=w='770+65*min(t/2.5,1)':h=-1:eval=frame,fade=t=in:st=0.10:d=0.65:alpha=1[logo];
[bg][logo]overlay=x='(W-w)/2':y='(H-h)/2-95':format=auto,
drawtext=fontfile='C\:/Windows/Fonts/seguisb.ttf':text='YANGI O‘ZBEK ALIFBOSI':fontcolor=0x081A42@0.82:fontsize=30:x=(w-text_w)/2:y=1100:alpha='min(1,max(0,(t-0.75)/0.65))',
drawbox=x='(w-420)/2':y=1160:w='420*min(1,max(0,(t-1.0)/0.8))':h=4:color=0x00CAD4@0.88:t=fill,
drawtext=fontfile='C\:/Windows/Fonts/segoeui.ttf':text='BITG':fontcolor=0x081A42@0.38:fontsize=21:x=(w-text_w)/2:y=1210:alpha='min(1,max(0,(t-1.25)/0.6))'[out]
'@
$scene5Args = @(
    "-y", "-f", "lavfi", "-i", "gradients=s=1080x1920:r=30:c0=0xFFFFFF:c1=0xEDF6FF:c2=0xB8F0F2:n=3:type=radial:speed=0.022:d=$sceneDuration",
    "-loop", "1", "-framerate", "30", "-i", $LogoPath,
    "-filter_complex", $scene5Filter, "-map", "[out]", "-t", $sceneDuration
) + $renderArgs + @($scene5)
Invoke-Ffmpeg $scene5Args

$silentVideo = Join-Path $buildDir "bitg-teaser-silent.mp4"
$xfade = @'
[0:v][1:v]xfade=transition=smoothleft:duration=0.35:offset=2.85[v01];
[v01][2:v]xfade=transition=fade:duration=0.35:offset=5.70[v02];
[v02][3:v]xfade=transition=radial:duration=0.35:offset=8.55[v03];
[v03][4:v]xfade=transition=fade:duration=0.35:offset=11.40,format=yuv420p[out]
'@
Invoke-Ffmpeg @(
    "-y", "-i", $scene1, "-i", $scene2, "-i", $scene3, "-i", $scene4, "-i", $scene5,
    "-filter_complex", $xfade, "-map", "[out]", "-c:v", "libx264", "-preset", "medium", "-crf", "18", "-pix_fmt", "yuv420p", "-r", "30", $silentVideo
)

$audioPath = Join-Path $buildDir "bitg-teaser-audio.wav"
$audioFilter = @'
[0:a]volume=0.32,afade=t=in:st=0:d=0.9,afade=t=out:st=13.7:d=0.9[amb];
[1:a]highpass=f=320,lowpass=f=5600,volume=0.16,afade=t=in:st=0:d=0.10,afade=t=out:st=0.18:d=0.42,asplit=4[w0][w1][w2][w3];
[w0]adelay=2500|2500[wd0];[w1]adelay=5350|5350[wd1];[w2]adelay=8200|8200[wd2];[w3]adelay=11050|11050[wd3];
[2:a]volume=0.25,afade=t=out:st=0.04:d=0.42,asplit=4[h0][h1][h2][h3];
[h0]adelay=2850|2850[hd0];[h1]adelay=5700|5700[hd1];[h2]adelay=8550|8550[hd2];[h3]adelay=11400|11400[hd3];
[3:a]volume=0.085,afade=t=out:st=0.12:d=1.05,adelay=11400|11400[ch];
[amb][wd0][wd1][wd2][wd3][hd0][hd1][hd2][hd3][ch]amix=inputs=10:normalize=0,volume=8,alimiter=limit=0.88[out]
'@
Invoke-Ffmpeg @(
    "-y",
    "-f", "lavfi", "-i", "aevalsrc=0.035*sin(2*PI*52*t)+0.018*sin(2*PI*104*t):s=48000:d=14.6",
    "-f", "lavfi", "-i", "anoisesrc=color=pink:amplitude=0.22:s=48000:d=0.60",
    "-f", "lavfi", "-i", "sine=frequency=72:sample_rate=48000:duration=0.46",
    "-f", "lavfi", "-i", "sine=frequency=660:sample_rate=48000:duration=1.20",
    "-filter_complex", $audioFilter, "-map", "[out]", "-c:a", "pcm_s16le", $audioPath
)

Invoke-Ffmpeg @(
    "-y", "-i", $silentVideo, "-i", $audioPath,
    "-map", "0:v:0", "-map", "1:a:0", "-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-shortest", "-movflags", "+faststart", $OutputPath
)

Write-Output "Created $OutputPath"
