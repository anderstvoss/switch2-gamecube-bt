[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$ResultFile,

    [Parameter(Mandatory, ValueFromRemainingArguments)]
    [string[]]$S2btArguments
)

$ErrorActionPreference = 'Stop'

$package = Get-AppxPackage -Name 'AndersVoss.Switch2GamecubeBt'
if ($null -eq $package) {
    throw 'The local Switch 2 GameCube Bluetooth capability host is not installed.'
}

$appUserModelId = "$($package.PackageFamilyName)!s2bt"
$escapedResultFile = $ResultFile.Replace('"', '\"')
$arguments = "--json --result-file `"$escapedResultFile`" $($S2btArguments -join ' ')"

if (-not ('Switch2GamecubeBt.PackageActivationManager' -as [type])) {
    Add-Type @'
using System;
using System.Runtime.InteropServices;

namespace Switch2GamecubeBt
{
    [ComImport, Guid("2e941141-7f97-4756-ba1d-9decde894a3d"),
     InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    internal interface IApplicationActivationManager
    {
        [PreserveSig]
        int ActivateApplication(
            [MarshalAs(UnmanagedType.LPWStr)] string appUserModelId,
            [MarshalAs(UnmanagedType.LPWStr)] string arguments,
            uint options,
            out uint processId);
    }

    [ComImport, Guid("45BA127D-10A8-46EA-8AB7-56EA9078943C")]
    internal class ApplicationActivationManager
    {
    }

    public static class PackageActivationManager
    {
        public static uint Activate(string appUserModelId, string arguments)
        {
            var manager = (IApplicationActivationManager)new ApplicationActivationManager();
            uint processId;
            var result = manager.ActivateApplication(appUserModelId, arguments, 0, out processId);
            Marshal.ThrowExceptionForHR(result);
            return processId;
        }
    }
}
'@
}

$processId = [Switch2GamecubeBt.PackageActivationManager]::Activate($appUserModelId, $arguments)
Wait-Process -Id $processId -Timeout 70

if (-not (Test-Path -LiteralPath $ResultFile -PathType Leaf)) {
    throw 'The packaged diagnostic exited without writing its sanitized result.'
}

Get-Content -LiteralPath $ResultFile -Raw
