!macro NSIS_HOOK_POSTINSTALL
  ; Refresh Explorer's icon for the replaced executable and its Start menu shortcut.
  ; SHCNE_UPDATEITEM with SHCNF_PATHW targets only the named item.
  System::Call 'shell32::SHChangeNotify(i 0x2000, i 0x0005, w "$INSTDIR\${MAINBINARYNAME}.exe", p 0)'
  ${If} ${FileExists} "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
    System::Call 'shell32::SHChangeNotify(i 0x2000, i 0x0005, w "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk", p 0)'
  ${EndIf}
!macroend

; Tauri removes the Run entry. Also clean up Windows' startup approval for Font Time.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run" "Font Time"
  ${EndIf}
!macroend
