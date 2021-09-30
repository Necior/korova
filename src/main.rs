use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use rand::seq::SliceRandom;

use serenity::model::id::ChannelId;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, user::User},
    prelude::*,
};

static ENVIRONMENT_VARIABLE_NAME: &str = "KOROVA_TOKEN";
static MIN_PLAYERS: usize = 2;

struct Handler;

struct ChannelGather {
    players: Vec<User>,
}

impl ChannelGather {
    fn new() -> Self {
        ChannelGather { players: vec![] }
    }

    fn add(&mut self, player: &User) {
        if self.players.iter().map(|p| p.id).all(|id| id != player.id) {
            self.players.push(player.clone());
        }
    }

    fn del(&mut self, player: &User) {
        self.players.retain(|p| p.id != player.id);
    }

    fn play(&mut self) -> String {
        if self.players.len() < MIN_PLAYERS {
            String::from("We need at least 2 players.")
        } else {
            let lines = vec![
                String::from("Get ready for the game. Let me summon everyone:"),
                self.players
                    .iter()
                    .map(|p| p.mention().to_string())
                    .collect::<Vec<_>>()
                    .join(" | "),
                String::from("Good luck & have fun!"),
            ];
            self.players = vec![];
            lines.join("\n")
        }
    }

    fn status(&self) -> String {
        if self.players.is_empty() {
            String::from("Nobody wants to play right now. Write `!add` to join.")
        } else {
            let mut lines = vec![
                String::from("Ready players:"),
                self.players
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<String>>()
                    .join(" | "),
            ];
            if self.players.len() >= MIN_PLAYERS {
                lines.push(String::from("Write `!play` to start the game."));
            }
            lines.join("\n")
        }
    }
}

struct GlobalGather;

impl TypeMapKey for GlobalGather {
    type Value = Arc<RwLock<HashMap<ChannelId, ChannelGather>>>;
}

/*
  Original author: Simon Travaglia
  Polish translation: Andrzej 'Greybrow' Korcala
  URL: http://greybrow.iq.pl/POPR/
  License: Creative Commons BY-NC-SA
*/
static EXCUSES: [&str; 466] = [
    "administrator jest od godziny zajęty usuwaniem prostej usterki",
    "administrator nie usłyszał telefonu z powodu prac budowlanych za oknem",
    "administrator pomylił taśmy do robienia kopii bezpieczeństwa",
    "administrator przypadkowo zniszczył telefon wielkim młotem",
    "administrator sieci rzucił pracę po zmuszeniu go do pracy z NT",
    "administrator stracił kontrolę nad siecią",
    "administratorzy porwani przez sektę",
    "administratorzy są nieobecni z powodu zebrania na temat, dlaczego są tak często nieobecni",
    "administratorzy zaatakowani przez wirusa roku 2000",
    "administratorzy zajęci zwalczaniem spamu",
    "aktualizacja /dev/null",
    "aktualizacja oprogramowania w ekspresie do kawy",
    "aktywność pozaziemska",
    "anomalia czasowa",
    "atak rewolucjonistów",
    "atak wirusa",
    "automatyczne korekcja ortografii w źródłach systemu operacyjnego",
    "automatyczny podajnik taśmy do kopii bezpieczeństwa wciągnął krawat administratora",
    "awaria awaryjnego wyłącznika prądu",
    "awaria klimatyzacji",
    "awaria systemu wysokiego ciśnienia",
    "baterie wysyłają zakłócające impulsy elektromagnetyczne",
    "błąd asynchroniczny",
    "błąd dzielenia przez zero",
    "błąd ID-10-T",
    "błąd konwersji",
    "błąd Pentium FDIV",
    "błąd przepełnienia w /dev/null",
    "błąd użyszkodnika",
    "błąd WE/WY",
    "błąd w oprogramowaniu",
    "błędy w danych testowych",
    "błędy w przepisywaniu nazw plików na serwerze",
    "Borg próbował zasymilować twój system. Opór jest bezcelowy",
    "brak aktualizacji vi do vii",
    "brak akumulatora w UPS",
    "brak internetu",
    "brak klawisza 'dowolny' na klawiaturze",
    "brak kursora",
    "brak licencji",
    "brak opłaty za wsparcie techniczne",
    "brak optymalizacji dysku twardego",
    "brak paliwa w agregacie prądotwórczym",
    "brak wolnych slotów na serwerze",
    "brak wykwalifikowanego elektryka do wymiany żarówki",
    "brak wymówki, wymyśl sam",
    "brak złącza RTFM",
    "cały Windows to ogólny błąd ochrony",
    "centrala przeszła na tonowe wybieranie",
    "chwilowe zakłócenie pracy protokołu sieciowego",
    "cięcia budżetowe",
    "cięcia budżetowe zmusiły do sprzedania części kabli sieciowych",
    "czasowe zakłócenia przesyłu danych",
    "cząsteczki promieniowania kosmicznego",
    "częstotliwość zegara",
    "dane intranetowe przypadkowe przeniesione do internetu",
    "dane na twoim dysku straciły równowagę",
    "dekoherencja kwantowa",
    "demon poczty pali wiadomości",
    "/dev/clue podlinkowany do /dev/null",
    "dodatkowe możliwości",
    "dostarczona instrukcja ma 34257 stron",
    "dostawca internetu ma problemy z siecią",
    "drukarka podpięta zamiast rutera",
    "duża aktywność nuklearna",
    "duża aktywność sejsmiczna",
    "duże wahania grawitacji",
    "dym z papierosa uruchomił system przeciwpożarowy",
    "dynamika kwantowa zaatakowała tranzystory",
    "dyski uszkodzone przez substancje z naklejek na dyskach",
    "dysk kręci się w odwrotną stronę",
    "dysk lub procesor się pali",
    "działa dokładnie tak jak to było zaprojektowane",
    "działa dokładnie tak jak to zrobili Chińczycy",
    "działalność terrorystyczna",
    "dzień wolny od pracy",
    "dziś urodziny Williama, więc wszystkie komputery z Windowsem świętują",
    "efekt Dopplera",
    "ekstradycja apletów Javy do Indonezji",
    "elektromagnetyczne promieniowanie kosmicznych śmieci",
    "elektromagnetyczne straty energii",
    "elektrownia ma problemy z reaktorem",
    "elektrownia testuje nowe urządzenia do zasilania",
    "elektrycy robili popcorn w zasilaczu",
    "elektryk nie wiedział do czego służy żółty kabelek i teraz nie ma sieci",
    "emisje tachionowe przegrzały system",
    "entropia",
    "erupcje słoneczne",
    "fala elektromagnetyczna spowodowana testowaniem nowej broni",
    "fałszujący modem",
    "FCC nie wydało zezwolenia",
    "firewall wymaga lepszego chłodzenia",
    "fundamenty budynku zostały postawione na starej elektrowni atomowej",
    "globalne ocieplenie",
    "głupi terminal",
    "gniazdo pluskiew spowodowało spięcie na kablu sieciowym",
    "greenpeace uwalniania komórki pamięci",
    "gremliny atakują",
    "hasło jest zbyt złożone by je odszyfrować",
    "HTTPD błąd 4004: zbyt stary procesor, brak odpowiedniej mocy obliczniowej",
    "HTTPD błąd 666: POPR tu był",
    "hydraulik pomylił kabel sieciowy z rurą",
    "impulsy elektromagnetyczne pochodzące z prób jądrowych",
    "infiltracja telemetryczna",
    "informacja zastrzeżona",
    "instrukcja napisana po chińsku",
    "internet chwilowo zamknięty przez serwis techniczny",
    "internet jest skanowany programem antywirusowym, proszę czekać",
    "ISP nie unowocześnił systemu",
    "ISP strajkuje",
    "ISP unowocześnia system",
    "jądro systemu w trakcie kompilacji",
    "jakieś małe zwierzątko wpadło do zasilacza",
    "ja nie admin, ja zmywać podłoga",
    "jeden z użytkowników używał serwera do kolportażu pornografi, serwer został przejęty przez policję",
    "jesteśmy z Apple, twój sprzęt wcale nie ma błędów",
    "jesteśmy z Microsoftu, twoje oprogramowanie wcale nie ma błędów",
    "jonizacja spowodowana klimatyzacją",
    "jony negatywne generowane przez światła fluorescencyjne",
    "jutro wygasa gwarancja na sprzęt",
    "już dostaliśmy takie zgłoszenie",
    "kabel do klawiatury traci przewodność elektryczną",
    "kable różnej długości",
    "kleistość",
    "klej z karteczek post-it przeciekł do monitora",
    "koła wycięte w polach kukurydzy",
    "kolizje pakietów sieciowych",
    "kompletne chwilowe wyłączenie",
    "komputer cierpi na zaniki pamięci",
    "komputer nie jest dobrze uziemiony (trzeba go zakopać)",
    "komputer nie zwraca wszystkich bitów",
    "komputery mają zbyt małą moc oliczeniową",
    "komputery pod wodą z powodu zalewu poczty",
    "koncentracja koncentratora",
    "koncert rockowy spowodował fluktuacje napięcia w sieci",
    "kondensacja pary w chmurze obliczeniowej",
    "konieczność restartu systemu",
    "korek w sieci",
    "kot próbował zjeść mysz",
    "kot rzucił się na mysz",
    "krawat wciągnięty przez drukarkę",
    "krzywa wiedzy użyszkodników okazała się być fraktalem",
    "ktoś obliczał liczbę pi na serwerze",
    "ktoś podpiął kable zasilania pod automatyczną sekretarkę",
    "ktoś pomyślał, że ten DUŻY CZERWONY GUZIK to włącznik światła",
    "ktoś się włamał do serwera i zastąpił system grą telewizyjną",
    "ktoś stanął na kablu sieciowym powodując zakłócenia",
    "ktoś ukradł kabel zasilania do serwera",
    "ktoś wrzucał listy do stacji dyskietek",
    "ktoś zakleił gniazdo interface'u gumą do żucia",
    "laser blueray wypalił dziurę w instalacji gazowej",
    "łatwiejsza praca, lepsza zabawa",
    "licencja nie obejmuje tego programu",
    "literówka w kodzie źródłowym",
    "ludzie z (dowolne miasto) zapychają łącza",
    "manipulator cyfrowy przekroczył parametry szybkości",
    "meteoryt uderzył w serwerownię",
    "miesięczna quota transferu wyczerpana przez pornografię",
    "mikroelektroniczne zakrzywienie przestrzenie Riemanna",
    "monitor podpięty do portu szeregowego",
    "mysz uszkodzona przez błąd-braku-sera",
    "myszy przegryzły kable zasilania",
    "myszy zrobiły powstanie i uciekły",
    "na administratorze zastosowano wolkański chwyt śmierci",
    "nanoroboty zainfekowały serwer",
    "napięcie statyczne",
    "napięcie statyczne w kablach sieciowych",
    "nawiedzony serwer",
    "niedziałająca sieć z powodu dostarczania pakietów IP przez pocztę",
    "niedziałający proces",
    "nie ma żadnego problemu",
    "nie odpowiada serwer domen",
    "nie płacone rachunki",
    "nie pytaj co administrator może zrobić dla ciebie, zastanów się co administrator może zrobić tobie",
    "nieuaktualniony system operacyjny",
    "nie włączone do prądu",
    "niewłaściwy czas synchronizacji",
    "niewystarczająca ilość pamięci",
    "niezaimplementowana funkcja",
    "niezgodność oprogramowania",
    "niezgodność rejestracji bitów",
    "niezgodność sprzętowa",
    "niezgodność z RFC-822",
    "nowe szefostwo",
    "nowy podpiął zasilanie do linii telefonicznej",
    "nowy sprzęt wymaga gęstszej sieci",
    "NWM (nie włączone myślenie)",
    "NWMU (nie włączony mózg użytkownika)",
    "obsługujemy tylko modemy starszej generacji",
    "odcięcie prądu w metrze",
    "odcięto internet podczas poszukiwań robaka",
    "odmowa dostępu",
    "odmowa połączenia telnetem",
    "odrzucanie uszkodzonych pakietów sieciowych",
    "odwrotnie podłączony router",
    "odwrotnie włożony dysk",
    "ograniczenie liczby jednocześnie obsługiwanych użytkowników",
    "operacja nie powiodła się z powodu braku komunikatu",
    "opóźnienia w rozsyłaniu",
    "oprogramowanie tak obciąża procesor, że system nie może się podnieść",
    "oprogramowanie używa innej miary niż system operacyjny",
    "oprogramowanie zbyt skomplikowane dla procesora",
    "oscylacje astropneumatyczne w chłodzeniu wodnym procesora",
    "ostatnia burza",
    "otrzymano bit stopu",
    "padł rdzeń",
    "pakiety danych zatrzymane na odprawie celnej",
    "pakiety nadawane ze złą częstotliwością",
    "pakiety pochłonięte przez terminator",
    "pakiety sieciowe odłożone na półkę",
    "pchły przegryzły kabel sieciowy",
    "pękła opona w wózku do przewozu taśm z kopiami bezpieczeństwa",
    "pękła rura z wodą przebiegająca nad salą z komputerami",
    "pierwsza sobota po pierwszej pełni księżyca w zimie",
    "PMKAK (problem między klawiaturą a krzesłem)",
    "poczta",
    "poczta elektroniczna dochodzi przez kraje, w których jest cenzura",
    "poczta elektroniczna dostarczana przez zwykłą pocztę",
    "podejrzany wskaźnik zawiesił maszynę wirtualną",
    "podłożona bomba",
    "podmieniony kod źródłowy",
    "podpięto żywą mysz do komputera",
    "pogryzione ciasteczka przeglądarki internetowej",
    "policja przechwytuje wszystkie pakiety w poszukiwaniu narkotyków",
    "poprzestawiane wskaźniki w jądrze systemu",
    "popsuty kondensator",
    "porzebny jest kod deszyfrujący od producenta oprogramowania",
    "potrzeba 10 razy szybszego sprzętu",
    "potrzeba podnieść serwer z depresji",
    "powódź uszkodziła połączenie z internetem",
    "powolność systemu ze względu na zły stan zasilania",
    "powolny przesył pakietów sieciowych",
    "pozostałości po próbach nuklearnych",
    "prąd statyczny z nylonowej bielizny",
    "prąd statyczny z plastikowych linijek",
    "problem zgodności z POSIX",
    "próby się nie udały, trzeba przeprojektować system",
    "proces niezgodny z ISO 9000",
    "procesor obsługuje zbyt wiele zadań",
    "procesor się zdecentralizował",
    "procesor wymaga rekalibracji",
    "procesor wymaga zmiany położenia",
    "procesory pracują prostopadle zamiast równolegle",
    "procesor za słabo się grzeje",
    "program do archiwizacji znowu zawiesił serwer",
    "projekt wojen gwiezdnych przypadkowo uszkodził satelitę komunikacyjnego",
    "promień cząsteczek alfa uszkodził pamięć ROM w serwerze",
    "proszę czekać...",
    "przekłamania w protokole sieciowym",
    "przekroczona liczba użyszkodników internetu",
    "przeładowanie bitami",
    "przeładowanie neutrinami",
    "przenosimy komputery do innego pomieszczenia",
    "przepełniona pamięć tylko do odczytu",
    "przepełniony /dev/null",
    "przerwania przestały przerywać",
    "przerwania w zasilaczu bezprzerwowym",
    "przeskoki bitów w pamięciach",
    "przeterminowane łącza SCSI",
    "przyjmujemy tylko zgłoszenia od licencjonowanych użytkowników",
    "przypadkowo uruchomione systemy halonowe zabiły administratorów",
    "przywrócono starą wersję systemu",
    "pseudo użytkownik przy pseudo terminalu",
    "psy ciemności zahipnotyzowały nocną zmianę",
    "ptaki obsiadły linie wysokiego napięcia",
    "/pub/piwo",
    "radiacja fraktalna zakłóca pracę rdzenia",
    "ręcznie kierowany satelita",
    "rekalibracja endotermiczna",
    "rekursywne pętle montowania dysków",
    "robactwo w macierzy dyskowej",
    "root utracił korzenie",
    "rozdwojenie jądra systemu",
    "rozproszenie promieniowania cieplnego",
    "ruter podpięty zamiast drukarki",
    "rytmiczne zakłócenia napięcia docierające do zasilacza",
    "satelita komunikacyjny został przejęty przez wojsko",
    "sekretarka włączyła suszarkę do włosów do UPS-a",
    "sekretarka wysłała łańcuszek szczęścia do wszystkich 5000 użytkowników w firmie",
    "serwer pocztowy zaatakowany przez łasice",
    "serwer pocztowy zaatakowany przez spamerów",
    "serwer popadł w schizofrenię",
    "serwer proxy",
    "serwer wymaga rekalibracji",
    "serwery domagają się ośmiogodzinnego dnia pracy",
    "serwery wypadły z synchronizacji",
    "serwer zakochał się w drukarce",
    "sieć wymaga aktualizacji",
    "skolioza rdzeni",
    "skradziono hasło administratora",
    "skradziony adres IP",
    "smród z pomieszczenia woźnego uszkodził taśmy z kopiami bezpieczeństwa",
    "spacja na klawiaturze generuje przypadkowe naciśnięcia klawiszy",
    "spalony koprocesor",
    "sprzedawca sprzedał sprzęt, którego nie ma w sprzedaży",
    "sprzedawca sprzedał zły towar",
    "sprzedawca uszkodził płytę główną",
    "standardowe wyłączenie systemu w celu konserwacji",
    "sterownik PCMCIA",
    "stłuczone okna",
    "stopione przewodniki elektryczne w komputerze",
    "strajk administratorów z powodu popsutego ekspresu do kawy",
    "strajk krasnoludków",
    "stres technologiczny",
    "stronicowanie pamięci zużyło papier do drukarki",
    "suche wtyki na wtyczce",
    "syn prezesa spieprzył komputer",
    "system operacyjny przerzuca dane na dysk",
    "system plików jest pełen plików",
    "system plików zbyt rozbudowany dla nowego jądra systemu",
    "szef zakazał administratorom zbliżać się do komputerów",
    "szef zapomniał hasła do systemu",
    "szkodliwe gazy ze zużytych nabojów do drukarek atramentowych",
    "tablica partycji uszkodzona przez korniki",
    "ta opcja nie była testowana",
    "TCP/IP UDP alarm spowodowany zbyt niskim poziomem granicy błędu",
    "technicy ze wsparcia technicznego mają kaca",
    "tej funkcji jeszcze nie ma, ale na pewno będzie w następnej wersji",
    "telefon od szefa",
    "temperatura procesora podniesiona przez wirusa",
    "ten rodzaj pamięci jest już nie używany",
    "terroryści rozbili samolot w serwerowni",
    "to będzie działać dopiero po następnym unowocześnieniu systemu",
    "to brzmi jak problem z Windowsem, proszę zadzwonić do Microsoftu",
    "to działa tylko w fińskiej wersji programu",
    "to działa tylko z nową łatą na system",
    "to nie działa z nowym uaktualnieniem systemu",
    "to nie nasz problem",
    "to przez chochliki",
    "to ty jesteś temu winien!",
    "to wszystko przez krasnoludki",
    "trzeba obrócić kryształy dwulitu",
    "twój komputer jest zbyt nowoczesny",
    "twój modem nie mówi po angielsku",
    "uderzenie pioruna",
    "układ zabezpieczający spalił układy zabezpieczane",
    "uparty proces",
    "UPS przerwał połączenie serwera z siecią",
    "UPS'y strajkują",
    "USS (użyszkodnik spieprzył sprawę)",
    "uszkodzenia wszczepów borga",
    "uszkodzenie chłodzenia procesora",
    "uszkodzenie pakietów spowodowane poplątanym okablowaniem",
    "uszkodzenie przesyłu pozytonów",
    "uszkodzenie starszych bajtów",
    "uszkodzenie systemu samonaprawiającego",
    "uszkodzenie wyłącznika sieciowego",
    "uszkodzona linia telefoniczna, nic nie słychać",
    "uszkodzona pamięć podręczna",
    "uszkodzone główne serwery DNS",
    "uszkodzone oczka sieci",
    "uszkodzone płaty wirnika w wentylatorze procesora",
    "USZKODZONE_RPC_PMAP",
    "uszkodzony dysk",
    "uszkodzony wieszak w tablicy partycji",
    "uwolniony bit parzystości",
    "uwolniony demon poczty",
    "użytkownicy grają w gry sieciowe",
    "wcześniej wszystko działało",
    "według Microsoftu to tak miało działać",
    "wentylator powoduje spięcia na procesorze",
    "węzeł na kablu spowodował pokręcenie strumienia danych",
    "wina internetu",
    "Windows",
    "wirus komputerowy przeniósł się na administratorów",
    "wirus spowodowany kontaktem z komputerem bez zabezpieczeń",
    "wirus w plikach aplikacji",
    "właśnie przeszliśmy na FDDI",
    "właśnie zmieniliśmy dostawcę internetu",
    "wpływ innego systemu operacyjnego",
    "w środku jest Intel",
    "wszyscy administratorzy są w szpitalu z powodu zatrucia pokarmowego",
    "wszystkie pakiety są puste",
    "wybiła studzienka i zalało serwerownię",
    "wybuchła podstacja elektrowni w parku",
    "wyciek eteru w sieci ethernetowej",
    "wyciek z monitora",
    "wygasła licencja na jądro systemu",
    "wykryto włamanie do systemu sprzed trzech miesięcy",
    "wyłączona opcja łączenia zdalnego",
    "wyłączyliśmy tę usługę z powodu cięć budżetowych",
    "wymiana awaryjnego wyłącznika prądu",
    "wymiana niewymiennego dysku",
    "wysoka oporność kabli",
    "wzrost radioaktywności",
    "zabrakło ci pamięci",
    "zabrakło kopert w serwerze poczty",
    "zabrakło nam bitów",
    "za duże oczka w sieci przepuszczają pakiety",
    "zaginione pakiety danych",
    "zajęte linie telefoniczne",
    "zakażenie wirusem z teleskopu Hubble",
    "zakłócenia magnetyczne spowodowane kartami kredytowymi",
    "zakłócenia piezoelektryczne",
    "zakłócenia przesyłu w sieci neuronowej",
    "zakłócenia przez podsłuch na liniach telefonicznych",
    "zakłócenia przez telefony komórkowe",
    "zakłócenia spowodowane gazem z zapalniczek",
    "zakłócenia spowodowane napięciem statycznym między biurkiem, a klawiaturą",
    "zakłócenia spowodowane promieniowaniem księżyca",
    "zakłócenia z pasa asteroidów",
    "zakłócenie przepływu plazmy",
    "zakrzywienie toru lotu elektronów",
    "załamanie nerwowe osprzętu",
    "za mała rozdzielczość monitora",
    "za mało przerwań",
    "zamarznięte obwody",
    "zamokły linie telefoniczne",
    "za niskie napięcie na płycie głównej",
    "zapchało się łącze o największej przepustowości",
    "zapchał się odpływ sieci, potrzebny hydraulik",
    "zaplanowane wyłączenie komputerów",
    "zapowiedziany koniec świata",
    "zaprojektowane ograniczenie",
    "za szybkie dyski SCSI",
    "zatarł się procesor",
    "za wąska szyna systemowa",
    "zawiesił się program odwieszający",
    "zbiegły koń trojański",
    "zbyt duża rozdzielczość ekranu",
    "zbyt duże promieniowanie wychodzące z ziemi",
    "zbyt duże zabezpieczenie przeciwprzepięciowe",
    "zbyt dużo nóżek na procesorze",
    "zbyt krótkie pakiety sieciowe",
    "zbyt wiele przerwań",
    "zbyt wielu użytkowników podłączyło się do serwera",
    "zbyt wolny interface SCSI",
    "zbyt wysokie napięci w sieci",
    "zdechł chomik biegający w plastikowym kółku",
    "zdezaktualizowane oprogramowanie, ale wciąż lepsze od nowego",
    "zdezaktualizowany serwer aktualizacji",
    "zepsute urządzenie odzyskiwania kopii zapasowych /dev/random",
    "zepsuty radiator procesora",
    "zepsuty włącznik klawiatury",
    "ze względu na cięcia budżetowe, dziś obsługujemy tylko konta na literę 'a'",
    "zimny lut",
    "zła polaryzacja przepływu neutronów",
    "źle określone okoliczności technicznego uszkodzenia",
    "źle podłączona klawiatura",
    "źle skonfigurowany przesył danych statycznych",
    "źle ustawiona klawiatura",
    "źle wskazywana pojemność dysku C:",
    "źle zamknięty system",
    "zły dowolny klawisz",
    "zły eter w sieci ethernetowej",
    "zły plik programu",
    "zły przydział numeru IP",
    "zły stan zasilania",
    "zły wpływ aury użytkownika",
    "zmiana kąta ustawienia procesora spowodowana wstrząsami z pobliskiej drogi",
    "zmiana nazwy serwera",
    "zmiana odcienia niebieskiego ekranu",
    "zmiana prędkości obrotowej Ziemi",
    "zmiana trybu wyświetlania z VESA na VISA",
    "zmiana wersji systemu operacyjnego",
    "zmiękczanie twardych dysków",
    "zmieniliśmy oficjalny język na COBOL",
    "znaleziono nieprawidłowego użytkownika",
    "znaleziono zapętloną pętlę w pętli sprzężenia zwrotnego",
    "zniszczenie konta roota",
    "zniszczona tablica połączeń oprogramowania dynamicznego",
    "zniszczone jądro systemu przy próbie kompilacji nowej wersji",
    "zrównanie Jowisza z Marsem",
    "zużyte sterowniki plików",
    "związki zawodowe",
    "zwiększony nacisk płyt tektonicznych",
];

fn get_weather() -> String {
    if let Ok(apikey) = env::var("KOROVA_OWM_APIKEY") {
        match &openweathermap::blocking::weather("Warsaw,PL", "metric", "pl", &apikey) {
            Ok(current) => {
                let desc = current.weather[0].description.to_string();
                let temp = format!("{}°C", current.main.temp);
                let pres = format!("{} hPa", current.main.pressure);
                format!("Pogoda w Warszawie: {}, {}, {}.", desc, temp, pres)
            }
            Err(e) => format!(
                "Coś się, coś się popsuło i nie było mnie słychać… (Informacja dla nerdów: {}.)",
                e
            ),
        }
    } else {
        "*chlip* *chlip*, jak mam sprawdzić pogodę, jeśli nie mam klucza do API?".to_string()
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let type_map = ctx.data.read().await;
        let lock = type_map.get::<GlobalGather>().unwrap().clone();
        let mut map = lock.write().await;
        let gather = map.entry(msg.channel_id).or_insert_with(ChannelGather::new);

        let response: Option<String> = match &msg.content[..] {
            "!add" => {
                gather.add(&msg.author);
                Some(gather.status())
            }
            "!del" => {
                gather.del(&msg.author);
                Some(gather.status())
            }
            "!play" => Some(gather.play()),
            "!status" => Some(gather.status()),
            "!help" => {
                let lines = vec![
                    "Gather commands: `!add`, `!del`, `!play`, `!status`.",
                    "Misc. commands: `!help`, `!ping`, `!weather`, `!wymówka`.",
                ];
                Some(lines.join("\n"))
            }
            "!ping" => Some(format!("Pong, {}.", msg.author.mention())),
            "!wymówka" => Some(format!(
                "{}",
                EXCUSES
                    .choose(&mut rand::thread_rng())
                    .unwrap_or(&"Pusta baza wymówek o_O")
            )),
            "!weather" => Some(get_weather()),
            _ => None,
        };

        if let Some(r) = response {
            if let Err(e) = msg.channel_id.say(&ctx.http, r).await {
                eprintln!("Error sending message: {:?}", e);
            }
        };
    }

    async fn ready(&self, _: Context, ready: Ready) {
        eprintln!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var(ENVIRONMENT_VARIABLE_NAME).unwrap_or_else(|_| {
        panic!(
            "Missing Discord bot token in {} environment variable.",
            ENVIRONMENT_VARIABLE_NAME
        )
    });

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<GlobalGather>(Arc::new(RwLock::new(HashMap::new())));
    }

    if let Err(e) = client.start().await {
        eprintln!("Client error: {:?}", e);
    }
}
