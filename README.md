# SLJ
Programski jezik v slovenščini | A programming language in Slovenian

SLJ je enostaven interpretiran programski jezik v slovenščini. Nastal je kot nadgradnja enostavnega kalkulatorja, razvil pa se je v nekaj precej zmogljivejšega.

Spodaj je primer uporabe jezika, v katerem je prisotna večina konstruktov jezika.

    funkcija je_deljivo(deljenec: celo, delitelj: celo) -> bool {
        vrni deljenec % delitelj == 0
    }

    funkcija je_praštevilo(kandidat: celo, praštevila: @[celo]) -> bool {
        za i = 0, praštevila[i]**2 <= kandidat, i += 1 {
            če je_deljivo(kandidat, praštevila[i]) {
                vrni laž
            }
        }
        vrni resnica
    }

    funkcija poišči_praštevila(od: celo, do: celo, praštevila: @[celo], praštevil: @celo) {
        za kandidat = od, kandidat <= do, kandidat += 2 {
            če je_praštevilo(kandidat, praštevila) {
                praštevila[praštevil@] = kandidat
                praštevil@ += 1
            }
        }
    }

    spr praštevila: [celo; 1_000_000]
    praštevila[0] = 2
    praštevila[1] = 3
    naj praštevil = 2


    poišči_praštevila(5, praštevila.dolžina, @praštevila, @praštevil)

    za i = 0, i < praštevil, i += 1 {
        natisni!(praštevila[i], " ")
    }

    natisni!("\npraštevil do ", praštevila.dolžina, ": ", praštevil, "\n")

# Konstrukti
## Spremenljivke in prirejanje
    spr x = 1
    x = x + 3.0 # Napaka E5: Nemogoča operacija: celo + real | test.slj:2:7
    x = x + 3

## Osnovni tipi
    spr odgovor: celo # cela števila
    spr pi: real      # realna števila
    spr enako: bool   # boolove vrednosti
    spr č: znak
    odgovor = 3.14 # Napaka E3: Nemogoča operacija: celo = real | test.slj:5:1
    odgovor = 42
    pi = 3.14
    enako = laž
    č = 'č'

## Reference
	naj a = 13
	naj ra = @a
	natisni(a)
	natisni(ra)  # Napaka E2: Funkcija 'natisni(@celo)' ne obstaja (5, 1)
	
	# dereferenciranje
	ra@ = 7
	natisni(ra@) # 7
	
## Seznami
	spr seznam: [real; 64]
	seznam[0] = 1 # Napaka E3: Nemogoča operacija: real = celo (2, 1)
	seznam[0] = 1.0
	seznam[1] = 2.0
	seznam[2] = 3.0

## Reference na sezname
Reference na sezname so enake za sezname vseh dolžin; seznama `[real; 13]` in `[real; 42]`  imata različna tipa, a imata oba referenco tipa `@[real]`.
Tako lahko implementiramo eno funkcijo za sezname vseh dolžin.

	funkcija vsebuje(seznam: @[real], št: real) -> bool {
	    za i = 0, i < seznam.dolžina, i += 1 {
	        če seznam[i] == št {
	            vrni resnica
	        }
	    }
	    vrni laž
	}
	
	spr a: [real; 13]
	spr b: [real; 42]
	spr c: [celo; 15]

	vsebuje(@a, 4.0)
	vsebuje(@b, 0.0)
	vsebuje(@c, -1) # Napaka E2: Funkcija 'vsebuje(@[celo], celo)' ne obstaja (16, 1)

## Operacije
	# aritmetične in bitne operacije
	spr x = 16 - 3
	x = x + 2
	x += 1
	x = 2**3
	x = 1 << 3
	x = x % 5
	x >>= 1 | 3
	
	# Boolove operacije
	spr b = laž
	b = !laž
	b = b && laž
	b ||= resnica
	b  ^= laž
	
	# pretvorbe med tipi
	spr a = 13.0
	a += 29 kot real
	spr nič = 48 kot znak
	nič += 11 kot znak # Napaka E3: Nemogoča operacija: znak += znak (4, 5)
	nič = (nič kot celo + 11) kot znak

	# primerjalne operacije
	naj enako = 13 == 11 + 2.0 # Napaka E5: Nemogoča operacija: celo + real (1, 22)
	naj večje = 13 > 14
	naj me    = 14 <= 14
	# ...
	
## Zanke
Zanka `dokler`:
	
	dokler seznam[0] {
	}
	# Napaka E6: Pogoj mora biti Boolova vrednost (1, 8)

	spr i = 0
	dokler seznam[i] > 0 {
		...
		i += 1
	}

Zanka `za`:

	za i = 0, i < seznam.dolžina, i += 1 {
		...
	}

## Pogojni stavki
	če št > 9 {
		...
	}
	čene če št > 0 {
		...
	}
	čene {
		...
	}

## Funkcije
	funkcija je_deljivo(deljenec: celo, delitelj: celo) {
	    vrni deljenec % delitelj == 0
	}
	# Napaka E3: Ne morem vrniti spremenljivke tipa 'bool' iz funkcije tipa 'brez' (2, 10)
	
	funkcija je_praštevilo(kandidat: celo, praštevila: @[celo]) -> bool {
	    za i = 0, praštevila[i]**2 < kandidat, i += 1 {
	        če je_deljivo(kandidat, i) {
	            vrni laž
	        }
	    }
	    vrni resnica
	}

## Multifunkcijski klic
Če obstaja več funkcij z enakim imenom, ki sprejemajo vsaka po en argument, lahko uporabimo multifunkcijski klic. Izgleda podobno kot navaden klic funkcije, le da za ime njeno dodamo `!`. Najboljši primer uporabe multifunkcijskih klicev je tiskanje.

	funkcija natisni(niz: @[znak]) {
	    naj dolžina = niz.dolžina
	    za i = 0, i < dolžina, i += 1 {
	        natisni(niz[i])
	    }
	}

	funkcija natisni(št: celo) {
	    če št > 9 {
	        natisni(št / 10)
	    }
	    # natisni(znak) je vprajena funkcija
	    natisni((št % 10 + '0' kot celo) kot znak)
	}

	natisni!("7! = ", faktoriela(7), "\n")

