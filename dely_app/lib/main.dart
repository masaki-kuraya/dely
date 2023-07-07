import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // このウィジェットはアプリケーションのルートです。
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
      theme: ThemeData(
        // これはアプリケーションのテーマです。
        //
        // "flutter run"でアプリケーションを実行してみてください。青いツールバーの
        // アプリケーションが表示されます。その後、アプリを終了せずに、以下の
        // primarySwatchをColors.greenに変更して、"hot reload"を実行してみてください。
        // (flutter runを実行したコンソールで "r" を押すか、Flutter IDEで変更を
        // 保存して "hot reload" を実行してください)。カウンターがゼロに戻らないことに
        // 注意してください。アプリケーションは再起動されません。
        colorSchemeSeed: Colors.deepPurple,
        useMaterial3: true,
      ),
      home: const MyHomePage(title: 'ホーム'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});

  // このウィジェットはアプリケーションのホームページです。ステートフルであり、
  // それは外観に影響を与えるフィールドを含む状態オブジェクトを持っています。
  //
  // このクラスは、状態の設定です。親から提供される値（この場合はAppウィジェット）
  // を保持し、状態のビルドメソッドで使用されます。ウィジェットのサブクラスのフィールドは
  // 常に "final" とマークされています。

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  final GlobalKey<ScaffoldState> _scaffoldKey = GlobalKey<ScaffoldState>();

  int _counter = 0;

  void _incrementCounter() {
    setState(() {
      // このsetStateの呼び出しは、この状態で何かが変更されたことをFlutterフレームワークに
      // 伝えます。これにより、下のビルドメソッドが再実行され、表示が更新された値を反映できるようになります。
      // setState()を呼ばずに_counterを変更した場合、ビルドメソッドが再度呼ばれず、
      // 何も起こらないように見えます。
      _counter++;
    });
  }

  @override
  Widget build(BuildContext context) {
    // setStateが呼ばれるたびに、たとえば上の_incrementCounterメソッドで行われるように、
    // このメソッドが再実行されます。
    //
    // Flutterフレームワークは、ビルドメソッドの再実行を高速化するために最適化されており、
    // 更新が必要なものをすべて再ビルドするだけで、ウィジェットの個々のインスタンスを
    // 変更する必要はありません。
    return Scaffold(
      key: _scaffoldKey,
      appBar: AppBar(
          // ここで、App.buildメソッドによって作成されたMyHomePageオブジェクトの値を取得し、
          // appbarのタイトルを設定します。
          title: Text(widget.title),
          centerTitle: false,
          actions: [
            IconButton(
              icon: const Icon(Icons.settings),
              onPressed: () {},
            ),
          ]),
      bottomNavigationBar: NavigationBar(
        destinations: const [
          NavigationDestination(
            icon: Icon(Icons.home),
            label: 'ホーム',
          ),
          NavigationDestination(
            icon: Icon(Icons.receipt),
            label: '予約',
          ),
          NavigationDestination(
            icon: Icon(
              Icons.calendar_today,
              size: 23,
            ),
            label: 'シフト',
          ),
          NavigationDestination(
            icon: Icon(Icons.face_3, size: 22),
            label: '女の子',
          ),
          NavigationDestination(
            icon: Icon(Icons.person, size: 27),
            label: 'お客様',
          ),
        ],
        selectedIndex: 0,
        onDestinationSelected: (int index) {
          setState(() {
            // TODO: ナビゲーションの選択を更新します。
          });
        },
      ),
      body: SingleChildScrollView(
        scrollDirection: Axis.vertical,
        child: Column(
          children: [
            Container(
              padding: const EdgeInsets.all(16),
              width: double.infinity,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  const Padding(
                    padding: EdgeInsets.only(bottom: 8),
                    child: Text(
                      "売上",
                      style: TextStyle(
                        fontSize: 16,
                      ),
                    ),
                  ),
                  Card(
                    elevation: 0,
                    color: Theme.of(context).colorScheme.surfaceVariant,
                    child: SizedBox(
                      width: double.infinity,
                      child: Container(
                          padding: const EdgeInsets.all(16),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.stretch,
                            children: [
                              Text('本日'),
                              Text('¥ 230,000',
                                  style: TextStyle(
                                    fontSize: 24,
                                  )),
                            ],
                          )),
                    ),
                  ),
                  Card(
                    elevation: 0,
                    color: Theme.of(context).colorScheme.surfaceVariant,
                    child: SizedBox(
                      width: double.infinity,
                      child: Container(
                          padding: const EdgeInsets.all(16),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.stretch,
                            children: [
                              Text('今月'),
                              Text('¥ 1000,000',
                                  style: TextStyle(
                                    fontSize: 24,
                                  )),
                            ],
                          )),
                    ),
                  ),
                ],
              ),
            )
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _incrementCounter,
        tooltip: '増加',
        child: const Icon(Icons.add),
      ), // この末尾のカンマは、ビルドメソッドの自動整形を綺麗にします。
    );
  }
}
