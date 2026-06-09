!include nsDialogs.nsh
!include LogicLib.nsh

!define MUI_FINISHPAGE_SHOWREADME_NOTCHECKED

Var DesktopShortcutCheckbox
Var DesktopShortcutSelection

Page custom DesktopShortcutOptionsPage DesktopShortcutOptionsPageLeave

Function DesktopShortcutOptionsPage
  IfSilent 0 +2
    Abort

  nsDialogs::Create 1018
  Pop $0
  ${If} $0 == error
    Abort
  ${EndIf}

  ${NSD_CreateLabel} 0 0 100% 24u "Choose whether GPU Tuner should create a desktop shortcut."
  Pop $0
  ${NSD_CreateCheckbox} 0 34u 100% 12u "Create desktop shortcut"
  Pop $DesktopShortcutCheckbox
  ${NSD_SetState} $DesktopShortcutCheckbox ${BST_CHECKED}
  StrCpy $DesktopShortcutSelection ${BST_CHECKED}

  nsDialogs::Show
FunctionEnd

Function DesktopShortcutOptionsPageLeave
  ${NSD_GetState} $DesktopShortcutCheckbox $DesktopShortcutSelection
FunctionEnd

!macro NSIS_HOOK_PREINSTALL
  ${If} $DesktopShortcutSelection = ${BST_UNCHECKED}
    StrCpy $NoShortcutMode 1
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ${If} $DesktopShortcutSelection != ${BST_UNCHECKED}
    Call CreateOrUpdateDesktopShortcut
  ${EndIf}
!macroend
