using ReactiveUI;
using SpacetimeDB.Types;

namespace Client.ViewModels;

public class MainWindowViewModel : ViewModelBase
{
    public static MainWindowViewModel Instance { get; private set; } = null!;

    public ViewModelBase ContentViewModel
    {
        get;
        private set => this.RaiseAndSetIfChanged(ref field, value);
    }

    public List<string> Channels { get; } = [];

    public SpacetimeDB? SpacetimeDb { get; private set; }

    public MainWindowViewModel()
    {
        Instance = this;
        ContentViewModel = new MainViewModel();
    }

    public void InitSpacetimeDb()
    {
        SpacetimeDb ??= new SpacetimeDB(Callback, TickCallback, (connection, identity) =>
        {
            connection.SubscriptionBuilder().OnApplied(context => { }).Subscribe([
                $"SELECT channel.id, channel.name FROM channel INNER JOIN member ON channel.id = member.channel_id WHERE member.user_id = {identity}"
            ]);
        });
    }

    private void Callback(DbConnection connection)
    {
    }

    private void TickCallback(DbConnection connection)
    {
    }

    public override void OnClose()
    {
        ContentViewModel.OnClose();
        SpacetimeDb?.Stop();
    }
}