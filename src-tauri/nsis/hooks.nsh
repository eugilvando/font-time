; Tauri removes the Run entry. Also clean up Windows' startup approval for Font Time.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run" "Font Time"
  ${EndIf}
!macroend
