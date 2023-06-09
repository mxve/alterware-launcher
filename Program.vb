Imports System
Imports System.IO
Imports System.Net
Imports System.Net.Http

Module Program
    Private master As String = "https://master.alterware.dev"

    Sub download_and_run(game As String)
        Dim filename As String = game & ".exe"
        Dim remote_path As String = If(game = "iw4-sp", "/iw4/", "/iw5/")
        Console.WriteLine("Downloading " & game & "...")
        Using wc As New WebClient
            wc.DownloadFile(master & remote_path & filename, filename)
        End Using
        Console.WriteLine("Starting " & game)
        Dim p As Process = Process.Start(filename)
        p.WaitForExit()
        End
    End Sub

    Sub Main(args As String())
        Dim game As String
        Try
            game = args(0)
        Catch ex As Exception
            If File.Exists("iw4sp.exe") Or File.Exists("iw4mp.exe") Then
                game = "iw4-sp"
            ElseIf File.Exists("iw5sp.exe") Or File.Exists("iw5mp.exe") Or File.Exists("iw5mp_server.exe") Then
                game = "iw5-mod"
            Else
                Console.WriteLine("No game specified nor found in local directory")
                Console.ReadLine()
                Return
            End If
        End Try

        download_and_run(game)
    End Sub
End Module