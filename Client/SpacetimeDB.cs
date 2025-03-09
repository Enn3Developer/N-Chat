using SpacetimeDB;
using SpacetimeDB.Types;

namespace Client;

public static class SpacetimeDB
{
#if DEBUG
    private const string Host = "http://localhost:3000";
#else
    private const string Host = "https://spacetime.enn3.ovh";
#endif

    private const string DbName = "n_chat";

    private static Identity? _localIdentity;

    public static bool Initialized { get; private set; }
    public static CancellationTokenSource? CancellationTokenSource { get; private set; }
    public static Thread? Thread { get; private set; }
    public static DbConnection? Connection { get; private set; }

    public static void Init()
    {
        if (Initialized) return;

        AuthToken.Init(".spacetime_csharp_n_chat");
        Connection = ConnectToDB();
        RegisterCallbacks(Connection);
        CancellationTokenSource = new CancellationTokenSource();
        Thread = new Thread(() => ProcessThread(Connection, CancellationTokenSource.Token));
        Thread.Start();
        Initialized = true;
    }

    public static void Stop()
    {
        if (!Initialized) return;

        CancellationTokenSource!.Cancel();
        Thread!.Join();
    }

    private static DbConnection ConnectToDB()
    {
        var conn = DbConnection.Builder()
            .WithUri(Host)
            .WithModuleName(DbName)
            .WithToken(AuthToken.Token)
            .OnConnect(OnConnected)
            .OnConnectError(OnConnectError)
            .OnDisconnect(OnDisconnected)
            .Build();
        return conn;
    }

    private static void RegisterCallbacks(DbConnection connection)
    {
    }

    private static void ProcessThread(DbConnection conn, CancellationToken ct)
    {
        try
        {
            // loop until cancellation token
            while (!ct.IsCancellationRequested)
            {
                conn.FrameTick();
                // ProcessCommands(conn.Reducers);
                Thread.Sleep(100);
            }
        }
        finally
        {
            conn.Disconnect();
        }
    }

    private static void OnConnected(DbConnection connection, Identity identity, string authToken)
    {
        _localIdentity = identity;
        AuthToken.SaveToken(authToken);
    }

    private static void OnConnectError(Exception e)
    {
        Console.Write($"Error while connecting: {e}");
    }

    private static void OnDisconnected(DbConnection conn, Exception? e)
    {
        Console.Write(e != null ? $"Disconnected abnormally: {e}" : "Disconnected normally.");
    }
}