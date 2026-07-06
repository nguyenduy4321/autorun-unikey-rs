@echo off
title Add Vietnamese Keyboard Layout for testing
echo Adding Vietnamese keyboard layout (042A:0000042A)...
powershell -c "$l=Get-WinUserLanguageList;$l.Add('vi');$l[-1].InputMethodTips.RemoveRange(0, $l[-1].InputMethodTips.Count);$l[-1].InputMethodTips.add('042A:0000042A');Set-WinUserLanguageList -Force -LanguageList $l"
echo.
echo Done! Check your taskbar language icon (or press Win + Space).
pause
