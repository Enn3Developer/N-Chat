using SpacetimeDB;
using SpacetimeDB.Types;

namespace Client;

public class SpacetimeDB
{
#if DEBUG
    private const string Host = "http://localhost:3000";
#else
    private const string Host = "https://spacetime.enn3.ovh";
#endif
    private const string DbName = "n-chat";

    private Identity? _localIdentity;
    private readonly CancellationTokenSource _cancellationTokenSource;
    private readonly Thread _thread;
    private Action<DbConnection, Identity> _connectedCallback;

    public string Username = null!;

    public SpacetimeDB(Action<DbConnection> callback, Action<DbConnection> tickCallback,
        Action<DbConnection, Identity> connectedCallback)
    {
        AuthToken.Init(".spacetime_csharp_n_chat");
        var connection = ConnectToDb();
        _connectedCallback = connectedCallback;
        callback(connection);
        _cancellationTokenSource = new CancellationTokenSource();
        _thread = new Thread(() => ProcessThread(connection, tickCallback, _cancellationTokenSource.Token));
        _thread.Start();
    }

    public void Stop()
    {
        _cancellationTokenSource.Cancel();
        _thread.Join();
    }

    private DbConnection ConnectToDb()
    {
        var connection = DbConnection.Builder()
            .WithUri(Host)
            .WithModuleName(DbName)
            .WithToken(AuthToken.Token)
            .OnConnect(OnConnected)
            .OnConnectError(OnConnectError)
            .OnDisconnect(OnDisconnected)
            .Build();
        return connection;
    }

    private static void ProcessThread(DbConnection conn, Action<DbConnection> tickCallback, CancellationToken ct)
    {
        try
        {
            // loop until cancellation token
            while (!ct.IsCancellationRequested)
            {
                conn.FrameTick();
                tickCallback(conn);
                Thread.Sleep(100);
            }
        }
        finally
        {
            conn.Disconnect();
        }
    }

    private void OnConnected(DbConnection connection, Identity identity, string authToken)
    {
        _localIdentity = identity;
        AuthToken.SaveToken(authToken);
        Console.WriteLine("Connected");
        Username = $"User{Random.Shared.NextInt64(ushort.MaxValue)}";
        connection.Reducers.SetName(Username);
        _connectedCallback(connection, identity);
    }

    private void OnConnectError(Exception e)
    {
        Console.Write($"Error while connecting: {e}");
    }

    private void OnDisconnected(DbConnection conn, Exception? e)
    {
        Console.Write(e != null ? $"Disconnected abnormally: {e}" : "Disconnected normally.");
    }
}