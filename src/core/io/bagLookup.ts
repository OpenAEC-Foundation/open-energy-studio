/**
 * Address lookup in the BAG through the public PDOK services (no key needed):
 * the Locatieserver finds the address by postcode and house number, the BAG
 * OGC API gives the verblijfsobject (use, floor area) and its pand (year of
 * construction). Feedback 9 Oct 2026: "project opzetten verbeteren".
 *
 * The BAG floor area is the registered gebruiksoppervlakte; NTA 8800 asks for
 * A_g measured per NEN 8800 annex, so the survey keeps it as a value to check.
 */

const LOCATIESERVER = 'https://api.pdok.nl/bzk/locatieserver/search/v3_1/free';
const BAG_OGC = 'https://api.pdok.nl/kadaster/bag/ogc/v2/collections';

export interface BagAddress {
  /** As the BAG shows it, e.g. "Goejanverwelledijk 85, 2807CB Gouda". */
  display: string;
  street: string;
  houseNumber: string;
  /** House letter and addition together, e.g. "a" or "A-01". */
  addition: string;
  postcode: string;
  city: string;
  /** Identification of the verblijfsobject (adresseerbaar object). */
  bagObjectId: string;
  /** BAG gebruiksdoel, e.g. "woonfunctie". */
  purpose?: string;
  /** BAG gebruiksoppervlakte, m². */
  floorAreaM2?: number;
  /** Bouwjaar of the pand. */
  constructionYear?: number;
  /** Number of verblijfsobjecten in the pand (more than one: an apartment building or mixed). */
  unitsInBuilding?: number;
}

/** "2807 cb" → "2807CB"; null when it is no Dutch postcode. */
export function normalizePostcode(value: string): string | null {
  const compact = value.replace(/\s+/g, '').toUpperCase();
  return /^[1-9]\d{3}[A-Z]{2}$/.test(compact) ? compact : null;
}

interface LocatieDoc {
  weergavenaam?: string; straatnaam?: string; huisnummer?: number; huisletter?: string; huisnummertoevoeging?: string;
  postcode?: string; woonplaatsnaam?: string; adresseerbaarobject_id?: string;
}

const additionOf = (doc: LocatieDoc) => [doc.huisletter, doc.huisnummertoevoeging].filter(Boolean).join('-');

/**
 * The addresses at a postcode and house number (several when there are
 * letters or additions), each with what the BAG says about it. An addition
 * given narrows the list to that one.
 */
export async function lookupAddresses(postcode: string, houseNumber: string, addition = '', fetcher: typeof fetch = fetch): Promise<BagAddress[]> {
  const code = normalizePostcode(postcode);
  const number = houseNumber.trim();
  if (!code || !/^\d+$/.test(number)) return [];
  const fields = 'weergavenaam,straatnaam,huisnummer,huisletter,huisnummertoevoeging,postcode,woonplaatsnaam,adresseerbaarobject_id';
  const url = `${LOCATIESERVER}?q=*&fq=type:adres&fq=postcode:${code}&fq=huisnummer:${number}&rows=20&fl=${fields}`;
  const response = await fetcher(url);
  if (!response.ok) throw new Error(`BAG ${response.status}`);
  const docs = ((await response.json()) as { response?: { docs?: LocatieDoc[] } }).response?.docs ?? [];
  const wanted = addition.trim().toLowerCase().replace(/[\s-]/g, '');
  const matching = wanted ? docs.filter((doc) => additionOf(doc).toLowerCase().replace(/-/g, '') === wanted) : docs;
  const addresses = matching.map((doc): BagAddress => ({
    display: doc.weergavenaam ?? '',
    street: doc.straatnaam ?? '',
    houseNumber: String(doc.huisnummer ?? number),
    addition: additionOf(doc),
    postcode: doc.postcode ?? code,
    city: doc.woonplaatsnaam ?? '',
    bagObjectId: doc.adresseerbaarobject_id ?? '',
  }));
  // Details only for a single match: the adviser picks first when there are several.
  if (addresses.length === 1) return [await withBuildingData(addresses[0], fetcher)];
  return addresses;
}

/** Adds use, floor area, year of construction and the number of units in the pand. */
export async function withBuildingData(address: BagAddress, fetcher: typeof fetch = fetch): Promise<BagAddress> {
  if (!address.bagObjectId) return address;
  try {
    const unit = await (await fetcher(`${BAG_OGC}/verblijfsobject/items?f=json&limit=1&identificatie=${address.bagObjectId}`)).json() as {
      features?: Array<{ properties?: Record<string, unknown> }>;
    };
    const props = unit.features?.[0]?.properties ?? {};
    const result: BagAddress = { ...address };
    if (typeof props.gebruiksdoel === 'string') result.purpose = props.gebruiksdoel;
    if (typeof props.oppervlakte === 'number') result.floorAreaM2 = props.oppervlakte;
    const href = props['pand.href'];
    const pandUrl = (Array.isArray(href) ? href[0] : href) as string | undefined;
    if (typeof pandUrl === 'string') {
      const pand = await (await fetcher(`${pandUrl.replace(/\?.*$/, '')}?f=json`)).json() as { properties?: Record<string, unknown> };
      if (typeof pand.properties?.bouwjaar === 'number') result.constructionYear = pand.properties.bouwjaar;
      if (typeof pand.properties?.aantal_verblijfsobjecten === 'number') result.unitsInBuilding = pand.properties.aantal_verblijfsobjecten;
    }
    return result;
  } catch {
    // The address stays usable without the building data.
    return address;
  }
}
