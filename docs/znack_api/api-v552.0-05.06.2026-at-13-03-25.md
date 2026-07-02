# Planned True API changes

Version 552.0

# **Contents**

| What's new in v.552.0 dd. May 29, 2026<br>4                                                         |
|-----------------------------------------------------------------------------------------------------|
| Planned changes<br>5                                                                                |
| ▪️ Working with industrial marking goods codes through the sub-account configuration                |
| mechanism<br>5                                                                                      |
| ▪️ Mandatory indication of previous code when remarking biologically active food additives<br>6     |
| ▪️ Specifying AVD when introducing confectionery products into circulation 6                        |
| ▪️ Specifying AVD when introducing semi-finished and frozen food products into circulation 7        |
| ▪️ Mandatory business place indication for withdrawal from circulation for certain goods groups 8   |
| ▪️ Adding and removing business places for engine oils 9                                            |
| ▪️ Simplified import process for laptops and smartphones from the Republic of Armenia<br>9          |
| ▪️ Migrating EDM Lite methods to True API<br>10                                                     |
| ▪️ Exporting codes by permitting document status groups and validity periods 15                     |
| ▪️ Exporting codes missing required permitting documents 17                                         |
| ▪️ Import of canned foods and seafood under mutual recognition of identification codes from the     |
| Republic of Belarus 19                                                                              |
| ▪️ Export of canned foods and seafood under mutual recognition of identification codes to the       |
| Republic of Belarus 20                                                                              |
| ▪️ Mandatory business place indication when withdrawing veterinary medicines from circulation<br>21 |
| ▪️ Universal messages in EDM Lite<br>22                                                             |
| ▪️ Method to create a universal message for an incoming / outgoing document<br>22                   |
| ▪️ Method to create a universal message for a buyer's title / cancellation proposal<br>26           |
| ▪️ Obtaining an authorization token in a new format for True API methods<br>29                      |
| ▪️ Import of biologically active food additives, veterinary medicines, perfume and eau de           |
| toilette, cosmetics, household chemicals, and personal hygiene products from the Republic of        |
| Armenia under mutual recognition of identification codes<br>34                                      |
| ▪️ Export of biologically active food additives, veterinary medicines, perfume and eau de           |
| toilette, cosmetics, household chemicals, and personal hygiene products to the Republic of          |
| Armenia 35                                                                                          |
| ▪️ Simplified process of import of engine oils from the Kyrgyz Republic<br>37                       |
| ▪️ Obtaining virtual warehouse information for canned foods<br>38                                   |
| ▪️ Mandatory indication of the date range when requesting a list of documents<br>38                 |
| ▪️ Filtering dairy products by shipment number in the virtual warehouse<br>39                       |
| ▪️ Manage consumer feedback<br>41                                                                   |
| ▪️ Method to get a list of consumer complaints 42                                                   |
| ▪️ Method to view consumer complaint<br>47                                                          |
| ▪️ Method to send a response to consumer complaint 51                                               |
| ▪️ Shipment of dairy products to the Republic of Belarus<br>53                                      |
| ▪️ Closing the "Withdrawal from circulation" document for export of light industry to the           |
| Republic of Belarus 54                                                                              |
| ▪️ Improvement of the method of receiving information on goods item by goods GTIN (v4) 55           |
| Appendix 1. Catalogs<br>57                                                                          |

| Catalog "List of supported goods groups"<br>57                                    |  |
|-----------------------------------------------------------------------------------|--|
| Catalog "Statuses of complaint consideration by goods circulation participant" 58 |  |
| Catalog "Types of consumer complaints" 59                                         |  |
| Catalog "Groups of permitting documents statuses"<br>59                           |  |
| Modifications introduced in the previous versions of the document<br>63           |  |

## • This document is for **informational purposes** and is intended to inform in advance the integrators and goods circulation participants about changes planned for implementation in True API.

#### **ATTENTION**

- **The final version of the changes** will become available after the announced features are implemented on the demo and production environments of the marking system.
- The changes described herein are planned for implementation in the next 1-2 months

# <span id="page-3-0"></span>**What's new in v.552.0 dd. May 29, 2026**

### Implemented:

▪️ New capabilities for managing the dairy product batch numbers: generation, reservation, and display.

For the details of implementation, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf) of v.672.0 dated May 29, 2026.

*To view a full document revision history, go to the "[Modifications introduced in the previous versions of](#page-62-0) [the document](#page-62-0)" section.*

# <span id="page-4-0"></span>**Planned changes**

## <span id="page-4-1"></span>**▪️ Working with industrial marking goods codes through the sub-account configuration mechanism**

#### • **Goods groups affected**

- Radio-electronic products (28 / radio)
- Radio-electronic products. Laptops and smartphones (51 / gadgets)

#### • **Business context**

Working with industrial marking (IM) goods codes with the 004 prefix will become available through the sub-account configuration mechanism as follows:

- companies that own IM goods codes will be able to grant full or restricted access to these codes;
- companies that are granted access to IM goods codes will be able to order marking codes for those codes and submit information about introduction into circulation.

When access to a goods code is granted, access to the packages associated with that goods item is automatically granted as well.

#### • **True API changes**

ὐ Methods to be updated:

- "Method of check of access of the sub-account to a list of goods codes" (/nk/linked-gtins)
- "Method of receiving a file in XML format to control access to sub accounts" (nk/linkedaccounts-documents)
- "Method of signing the granting or recalling the permission to use IC" (/nk/linkedaccounts-sign)

#### • **Details**:

You will be able to specify IM goods codes with the 004 prefix in the following methods:

- "Method of check of access of the sub-account to a list of goods codes" (/nk/linkedgtins) — in the gtin parameter (Goods code)
- "Method of receiving a file in XML format to control access to sub accounts" (nk/linkedaccounts-documents) — in the linked\_goods parameter (Array of goods codes to which access is being provided)

Using the "Method of signing the granting or recalling the permission to use IC" method

<span id="page-5-2"></span><span id="page-5-1"></span><span id="page-5-0"></span>(/nk/linked-accounts-sign), you will be able to sign an XML file containing IM goods codes with the 004 prefix. For the description of the methods, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf). **▪️ Mandatory indication of previous code when remarking biologically active food additives** • **Goods groups affected** ◦ Specialized food products and biologically active food additives (17 / bio) • **Business context** Indication of the previous identification code details will become mandatory when remarking goods. These changes are being implemented in accordance with [Decree of the Government of the](https://честныйзнак.рф/upload/Постановление_Правительства_Российской_Федерации_от_28_11_2025_№886.pdf) [Russian Federation dated November 28, 2025 No. 1953.](https://честныйзнак.рф/upload/Постановление_Правительства_Российской_Федерации_от_28_11_2025_№886.pdf) • **True API changes** ὐ The document to be updated: "Remarking" (LK\_REMARK, LK\_REMARK\_XML) • **Details** When submitting the "Remarking" document (LK\_REMARK, LK\_REMARK\_XML), the last\_uin parameter (Previous code) will become mandatory. For the description of the document, see [True API.](https://честныйзнак.рф/upload/docs/True_API_en.pdf) **▪️ Specifying AVD when introducing confectionery products into circulation** • **Goods groups affected** ◦ Confectionery products (45 / sweets) • **Business context** Specifying the AVD (accompanying veterinary document) identifier will become mandatory when introducing goods subject to veterinary controls into circulation. • **True API changes** ὐ Documents to be updated:

- "Introduction into circulation. Manufacturing in the Russian Federation" (LP\_INTRODUCE\_GOODS)
- "Introduction into circulation. Manufacturing outside of EEU" (LP\_GOODS\_IMPORT)
- "Introduction into circulation. Cross-border trading" (CROSSBORDER)
- "Introduction into circulation. Contract manufacturing" (LK\_CONTRACT\_COMMISSIONING)
- "Introduction into circulation. Import with FCS" (LP\_FTS\_INTRODUCE)
- "Acceptance of shipment from EEU" (LP\_ACCEPT\_GOODS)
- "Correction of information on codes" (CIS\_INFORMATION\_CHANGE)

#### • **Details**

In the [documents listed above,](#page-5-2) the vsd\_number parameter (AVD identifier) will become mandatory for goods subject to veterinary controls. In the "Correction of information on codes" document (CIS\_INFORMATION\_CHANGE), the vsd parameter (AVD identifier) will be used for this purpose.

For the description of the documents, see [True API.](https://честныйзнак.рф/upload/docs/True_API_en.pdf)

## <span id="page-6-0"></span>**▪️ Specifying AVD when introducing semi-finished and frozen food products into circulation**

#### • **Goods groups affected**

◦ Semi-finished and frozen food products (52 / frozen)

#### • **Business context**

Specifying the AVD (accompanying veterinary document) identifier will become mandatory when introducing goods subject to veterinary controls into circulation.

#### • **True API changes**

<span id="page-6-1"></span>ὐ Documents to be updated:

- "Introduction into circulation. Manufacturing in the Russian Federation" (LP\_INTRODUCE\_GOODS)
- "Introduction into circulation. Manufacturing outside of EEU" (LP\_GOODS\_IMPORT)
- "Introduction into circulation. Cross-border trading" (CROSSBORDER)
- "Introduction into circulation. Contract manufacturing" (LK\_CONTRACT\_COMMISSIONING)

#### • **Details**

In the [documents specified above,](#page-6-1) the vsd\_number parameter (AVD identifier) will become mandatory for goods subject to veterinary controls.

For the description of the documents, see [True API.](https://честныйзнак.рф/upload/docs/True_API_en.pdf)

## <span id="page-7-0"></span>**▪️ Mandatory business place indication for withdrawal from circulation for certain goods groups**

#### • **Goods groups affected**

- Perfume and eau de toilette (4 / perfumery)
- Games and toys for children (27 / toys)
- Light industry (1 / lp)
- Footwear (2 / shoes)
- Radio-electronic products. Laptops and smartphones (51 / gadgets)
- Building materials (39 / construction)
- New pneumatic rubber tires and tire casings (5 / tires)

### • **Business context**

Specifying the business place will become mandatory when withdrawing goods from circulation for certain withdrawal reasons. This change will improve the accuracy of goods movement tracking and ensure compliance with marking requirements.

#### • **True API changes**

ὐ The document to be updated: "Withdrawal from circulation" (LK\_RECEIPT)

#### • **Details**

Providing business place details will be required in the kpp (KPP of goods circulation participant) and fias\_id (FIAS identifier) parameters if the action parameter (Disposal reason) has one of the following values:

- DISTANCE (Distance selling)
- BY\_SAMPLES (Sale by sample)
- VENDING (Sale through a vending machine)

◦ RETAIL (Retail sale)

For the description of the document, see [True API.](https://честныйзнак.рф/upload/docs/True_API_en.pdf)

## <span id="page-8-0"></span>**▪️ Adding and removing business places for engine oils**

#### • **Goods groups affected**

◦ Engine oils (43 / autofluids)

#### • **Business context**

You will be able to add and remove business places. This update aims to maintain data accuracy in the system and ensure compliance with regulatory requirements.

#### • **True API changes**

✅ The methods to be available:

- "Method to add business places" (/api/v3/true-api/mods/add)
- "Method to remove business places" (/api/v3/true-api/mod)

#### • **Details**

For the method descriptions, see [True API.](https://честныйзнак.рф/upload/docs/True_API_en.pdf)

## <span id="page-8-1"></span>**▪️ Simplified import process for laptops and smartphones from the Republic of Armenia**

#### • **Goods groups affected**

◦ Radio-electronic products. Laptops and smartphones (51 / gadgets)

#### • **Business context**

A simplified process of import from the Republic of Armenia to the Russian Federation is planned for implementation.

The following scheme will be implemented to import goods under the simplified process:

- 1. An exporter from the Republic of Armenia describes a goods item in the National catalog and orders Russian marking codes via the Marking system of the country of export.
- 2. The exporter then applies the marking codes onto the goods and submits cross-border movement details to the Marking system of the Republic of Armenia. The importer can view

these details in the Marking system of the Russian Federation in the "Shipment from EEU" document (LP\_SHIP\_GOODS\_CROSSBORDER).

3. The importer submits the "Acceptance" document (LP\_ACCEPT\_GOODS) containing information on the acceptance of goods specified in the shipment document to the Marking system of the Russian Federation. After the acceptance document is successfully processed, the accepted marking codes will be introduced into circulation.

#### • **True API changes**

✅The document to be available: "Acceptance of shipment from EEU" (LP\_ACCEPT\_GOODS) submitted in response to the "Shipment from EEU" document (LP\_SHIP\_GOODS\_CROSSBORDER)

#### • **Details**

Parameter requirements for the "Acceptance of shipment from EEU" document ( LP\_ACCEPT\_GOODS) submitted in response to the "Shipment from EEU" document (LP\_SHIP\_GOODS\_CROSSBORDER) will match those for import of *laptops and smartphones* from the Republic of Belarus.

For the description of the documents, see [True API.](https://честныйзнак.рф/upload/docs/True_API_en.pdf)

## <span id="page-9-0"></span>**▪️ Migrating EDM Lite methods to True API**

#### • **Goods groups affected**

◦ All goods groups

#### • **Business context**

EDM Lite methods will become available via True API. This will simplify integration through a unified interface and authentication mechanism.

#### • **True API changes**

✅ EDM Lite methods will become available via True API.

#### • **Details**

Request and response parameters will remain unchanged. For more details of the methods, see ["Description of API of EDM Lite"](https://docs.crpt.ru/gismt/Документация_по_API_ЭДО_Лайт/).

| Method name                                                                                                                   | URL in EDM Lite<br>(before)                          | URL in True API<br>(after)                                           |
|-------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------|----------------------------------------------------------------------|
| Method to upload seller's details in UTD XML<br>file                                                                          | /api/v1/outgoing<br>documents                        | /api/v3/true<br>api/edo/outgoing<br>documents                        |
| Method to upload seller's details in UTD(c)<br>XML file                                                                       | /api/v1/outgoing<br>documents/xml/updi               | /api/v3/true<br>api/edo/outgoing<br>documents/xml/updi               |
| Method to upload seller's details in UCD XML<br>file in accordance with order 189 dated April 13,<br>2016                     | /api/v1/outgoing<br>documents/xml/ukd                | /api/v3/true<br>api/edo/outgoing<br>documents/xml/ukd                |
| Method to upload seller's details in UCD XML<br>file in accordance with order 736 dated October<br>12, 2020                   | api/v1/outgoing<br>documents/xml/ukd/7<br>36         | /api/v3/true<br>api/edo/outgoing<br>documents/xml/ukd/7<br>36        |
| Method to upload seller's details in UCD(c)<br>XML file in accordance with order 736 dated<br>October 12, 2020                | /api/v1/outgoing<br>documents/xml/ukdi/<br>736       | /api/v3/true<br>api/edo/outgoing<br>documents/xml/ukdi/<br>736       |
| Sign outgoing document                                                                                                        | /api/v1/outgoing<br>documents/{doc_id}/<br>signature | /api/v3/true<br>api/edo/outgoing<br>documents/{doc_id}/<br>signature |
| Obtain print form for outgoing UTD / UTD(c) /<br>UCD                                                                          | /api/v1/outgoing<br>documents/{doc_id}/<br>print     | /api/v3/true<br>api/edo/outgoing<br>documents/{doc_id}/<br>print     |
| Obtain print form for incoming UTD / UTD(c) /<br>UCD                                                                          | /api/v1/incoming<br>documents/{doc_id}/<br>print     | api/v3/true<br>api/edo/incoming<br>documents/{doc_id}/<br>print      |
| Method to upload buyer's details in UTD XML<br>file in accordance with order 820 dated<br>December 19, 2018 No. ММВ-7-15/820@ | api/v1/incoming<br>documents/xml/upd/t<br>itle       | /api/v3/true<br>api/edo/incoming<br>documents/xml/upd/t<br>itle      |

| Method name                                                                                                                      | URL in EDM Lite<br>(before)                          | URL in True API<br>(after)                                           |
|----------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------|----------------------------------------------------------------------|
| Method to upload buyer's details in UTD(c)<br>XML file in accordance with order 820 dated<br>December 19, 2018 No. ММВ-7-15/820@ | /api/v1/incoming<br>documents/xml/updi/<br>title     | /api/v3/true<br>api/edo/incoming<br>documents/xml/updi/<br>title     |
| Method to upload buyer's title in UTD XML file<br>in accordance with order No. ЕД-7-26/970@<br>dated December 20, 2023           | api/v1/incoming<br>documents/xml/upd/t<br>itle/970   | /api/v3/true<br>api/edo/incoming<br>documents/xml/upd/t<br>itle/970  |
| Method to upload buyer's title in UTD(c) XML<br>file in accordance with order No. ЕД-7-26/970@<br>dated December 20, 2023        | api/v1/incoming<br>documents/xml/updi/<br>title/970  | /api/v3/true<br>api/edo/incoming<br>documents/xml/updi/<br>title/970 |
| Method to upload buyer's details in UCD XML<br>file in accordance with order 189 dated April 13,<br>2016                         | /api/v1/incoming<br>documents/xml/ukd/t<br>itle      | /api/v3/true<br>api/edo/incoming<br>documents/xml/ukd/t<br>itle      |
| Method to upload buyer's details in UCD XML<br>file in accordance with order 736 dated October<br>12, 2020                       | /api/v1/incoming<br>documents/xml/ukd/t<br>itle/736  | /api/v3/true<br>api/edo/incoming<br>documents/xml/ukd/t<br>itle/736  |
| Method to upload buyer's details in UCD(c)<br>XML file in accordance with order 736 dated<br>October 12, 2020                    | /api/v1/incoming<br>documents/xml/ukdi/<br>title/736 | /api/v3/true<br>api/edo/incoming<br>documents/xml/ukdi/<br>title/736 |
| Get a list of receipts for outgoing documents                                                                                    | /api/v1/outgoing<br>documents/unsigned<br>event      | /api/v3/true<br>api/edo/outgoing<br>documents/unsigned<br>events     |
| Get a list of receipts for incoming documents                                                                                    | /api/v1/incoming<br>documents/unsigned<br>events     | /api/v3/true<br>api/edo/incoming<br>documents/unsigned<br>events     |

| Method name                                                             | URL in EDM Lite<br>(before)                                                | URL in True API<br>(after)                                                                          |
|-------------------------------------------------------------------------|----------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------|
| Get an XML file with buyer's details (UTD /<br>UTD(c) / UCD) or receipt | /api/v1/incoming<br>documents/{doc_id}/<br>events/{event_id}/c<br>ontent   | /api/v3/true<br>api/edo/incoming<br>documents/{doc_id}/<br>events/{event_id}/c<br>ontent            |
| Sign receipts or a file with buyer's details (UTD<br>/ UTD(c) / UCD)    | /api/v1/incoming<br>documents/{doc_id}/<br>events/{event_id}/s<br>ignature | /api/v3/true<br>api/edo/incoming<br>documents/{doc_id}/<br>events/{event_id}/s<br>ignature          |
| Create a notice of clarification by uploading an<br>XML file            | api/v1/incoming<br>documents/xml/uvtoc<br>h                                | /api/v3/true<br>api/edo/incoming<br>documents/xml/uvtoc<br>h                                        |
| Request cancellation of outgoing document by<br>uploading an XML file   | api/v1/outgoing<br>documents/xml/annul<br>_02                              | /api/v3/true<br>api/edo/outgoing<br>documents/xml/annul<br>_02                                      |
| Request cancellation of incoming document by<br>uploading an XML file   | api/v1/incoming<br>documents/xml/annul<br>_02                              | /api/v3/true<br>api/edo/incoming<br>documents/xml/annul<br>_02                                      |
| Accept cancellation for outgoing documents                              | /api/v1/outgoing<br>documents/{doc_id}/<br>events/{event_id}/s<br>ignature | /api/v3/true<br>api/edo/outgoing<br>documents/{doc_id}/<br>events/{event_id}/s<br>ignature          |
| Accept cancellation for incoming documents                              | /api/v1/incoming<br>documents/{doc_id}/<br>events/{event_id}/s<br>ignature | /api/v3/true<br>api/elk/incoming<br>documents/{doc_id}/<br>events/{event_id}/a<br>nnul_02/signature |

| Method name                                                                                              | URL in EDM Lite<br>(before)                                          | URL in True API<br>(after)                                                              |
|----------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------|-----------------------------------------------------------------------------------------|
| Reject cancellation of outgoing document by<br>uploading an UvToch XML file (Notice of<br>clarification) | api/v1/outgoing<br>documents/xml/annul<br>_uvtoch                    | /api/v3/true<br>api/edo/outgoing<br>documents/xml/annul<br>_uvtoch                      |
| Reject cancellation of incoming document by<br>uploading an UvToch XML file (Notice of<br>clarification) | api/v1/incoming<br>documents/xml/annul<br>_uvtoch                    | /api/v3/true<br>api/edo/incoming<br>documents/xml/annul<br>_uvtoch                      |
| Method to get a JSON receipt for outgoing<br>document receipt                                            | api/v1/outgoing<br>documents/document<br>ID/events/{receipt<br>type} | /api/v3/true<br>api/edo/outgoing<br>documents/{document<br>ID}/events/{receipt<br>type} |
| Method to get a JSON receipt for incoming<br>document receipt                                            | api/v1/incoming<br>documents/document<br>ID/events/{receipt<br>type} | /api/v3/true<br>api/edo/incoming<br>documents/{document<br>ID}/events/{receipt<br>type} |
| Method to create UTD(c) via XML                                                                          | api/v1/outgoing<br>documents/xml/updi/<br>970                        | /api/v3/true<br>api/edo/outgoing<br>documents/xml/updi/<br>970                          |
| Method to get data on machine-readable power<br>of attorney for outgoing documents                       | api/v1/outgoing<br>documents/{doc_id}/<br>mchd-list                  | /api/v3/true<br>api/edo/outgoing<br>documents/{doc_id}/<br>mchd-list                    |
| Method to get data on machine-readable power<br>of attorney for incoming documents                       | api/v1/incoming<br>documents/{doc_id}/<br>mchd-list                  | /api/v3/true<br>api/edo/incoming<br>documents/{doc_id}/<br>mchd-list                    |

## <span id="page-14-0"></span>**▪️ Exporting codes by permitting document status groups and validity periods**

#### • **Goods groups affected**

◦ All goods groups

#### • **Business context**

You will be able to export identification codes:

- by permitting document status groups
- by permitting document validity period

This will allow you to quickly determine which codes require permitting document updates and to avoid blocking.

How this will work:

- 1. Use the "Method to obtain a catalog of deviation categories and types" method (/api/v3/true-api/directory/statistics) to get the permitting document index from indx (Index of the permitting document).
- 2. Request a FILTERED\_CIS\_REPORT export using the "Method of receiving the list of GCP's identification codes by set filter" method (/dispenser/tasks), specifying the previously received index in the permitDocIndx parameter (Indexes of the permitting documents).

#### • **True API changes**

ὐ Methods to be updated:

- "Method to obtain a catalog of deviation categories and types" (/directory/statistics)
- "Method of receiving the list of GCP's identification codes by set filter" (/dispenser/tasks)

#### • **Details**

- The following will be added to the "Method to obtain a catalog of deviation categories and types" method (/directory/statistics):
  - a possible value for the data\_set parameter (Service name): RD\_STATUS\_BY\_INN (Permitting document status indicators);
  - optional request parameters for the filters array (Array of filters for received data):

| Parameter      | Type                 | Manda<br>tory | Description                                         | Comment                                                                                                                                                                   |
|----------------|----------------------|---------------|-----------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| status_groups  | array of<br>integers | -             | Array of<br>permitting<br>document status<br>groups | The parameter is filled in when<br>data_set is RD_STATUS_BY_INN.<br>Example: 3<br>—<br>red status.<br>See "Catalog "Groups of permitting<br>documents statuses""          |
| product_groups | array of<br>integers | -             | Array of goods<br>group codes                       | The parameter is filled in when<br>data_set is RD_STATUS_BY_INN.<br>Example: 2<br>—<br>footwear.<br>See "Catalog "List of supported<br>goods groups""                     |
| reg_end_date   | string<br>(date)     | -             | Permitting<br>document<br>expiration date           | The parameter is filled in when<br>data_set is RD_STATUS_BY_INN.<br>Format: yyyy-MM-dd. If the<br>parameter is not specified, the<br>default value is used: '1970-01-01'. |

#### ▪ optional response parameters:

| Parameter             | Type                | Manda<br>tory | Description                                           | Comment                                                                                                                   |
|-----------------------|---------------------|---------------|-------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------|
| rd_status_for_in<br>n | array of<br>objects | -             | Array of<br>permitting<br>document<br>statuses by INN | The parameter is returned when<br>data_set is RD_STATUS_BY_INN.                                                           |
| *rd_for_inn           | array of<br>objects | +             | List of<br>permitting<br>documents by<br>INN          |                                                                                                                           |
| **indx                | integer             | -             | Index of the<br>permitting<br>document                | The parameter is not returned if no<br>permitting documents with the<br>specified status and expiration date<br>are found |

| Parameter      | Type             | Manda<br>tory | Description                               | Comment |
|----------------|------------------|---------------|-------------------------------------------|---------|
| **status_group | integer          | -             | Permitting<br>document status<br>group    |         |
| **reg_end_date | string<br>(date) | -             | Permitting<br>document<br>expiration date |         |

◦ the following optional response parameter will be added to the "Method of receiving the list of GCP's identification codes by set filter" method (/dispenser/tasks):

| Parameter                | Type   | Mandat<br>ory | Description                            | Comment                                                                                                                                        |
|--------------------------|--------|---------------|----------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------|
| permitDocStatus<br>Group | string | -             | Permitting<br>document status<br>group | The parameter is returned if data are<br>present in a goods card in the<br>NCMG.<br>See "Catalog "Groups of permitting<br>documents statuses"" |

For the description of the methods, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf).

## <span id="page-16-0"></span>**▪️ Exporting codes missing required permitting documents**

### • **Goods groups affected**

◦ All goods groups

### • **Business context**

You will be able to export codes that are missing required permitting documents. This will help you identify which codes require permitting document registration and prevent errors in the future.

How this will work:

- 1. Use the "Method to obtain a catalog of deviation categories and types" method (/api/v3/true-api/directory/statistics) to get goods codes missing permitting documents via rd\_gtin\_for\_inn (List of goods codes without permitting documents).
- 2. Request a FILTERED\_CIS\_REPORT export using the "Method of receiving the list of GCP's

identification codes by set filter" method (/dispenser/tasks), specifying the previously received goods codes in the includeGtin parameter (Goods code for search).

#### • **True API changes**

ὐ Method to be updated:

◦ "Method to obtain a catalog of deviation categories and types" (/directory/statistics)

#### • **Details**

The following will be added to the "Method to obtain a catalog of deviation categories and types" method (/directory/statistics):

- a possible value for the data\_set parameter (Service name): MISSING\_RD\_BY\_INN (Permitting document absence indicators);
- optional request parameters for the filters array (Array of filters for received data):

| Parameter      | Type                 | Mandat<br>ory | Description                    | Comment                                                                                                                                           |
|----------------|----------------------|---------------|--------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------|
| product_groups | array of<br>integers | -             | Array of goods<br>groups codes | The parameter is used when<br>data_set is MISSING_RD_BY_INN.<br>Example: 2<br>—<br>footwear.<br>See "Catalog "List of supported<br>goods groups"" |

◦ optional response parameters:

| Parameter              | Type                | Mandat<br>ory | Description                                           | Comment                                                         |
|------------------------|---------------------|---------------|-------------------------------------------------------|-----------------------------------------------------------------|
| missing_rd_for_i<br>nn | array of<br>objects | -             | Array of missing<br>permitting<br>documents by<br>INN | The parameter is returned when<br>data_set is MISSING_RD_BY_INN |
| *rd_gtin_for_inn       | array of<br>objects | +             | List of GTINs<br>without<br>permitting<br>documents   |                                                                 |

| Parameter | Type   | Mandat<br>ory | Description    | Comment |
|-----------|--------|---------------|----------------|---------|
| **gtin    | string | -             | GTIN missing a |         |
|           |        |               | required       |         |
|           |        |               | permitting     |         |
|           |        |               | document       |         |

For the description of the method, see [True API.](https://честныйзнак.рф/upload/docs/True_API_en.pdf)

## <span id="page-18-0"></span>**▪️ Import of canned foods and seafood under mutual recognition of identification codes from the Republic of Belarus**

#### • **Goods groups affected**

- Canned foods (32 / conserve)
- Seafood (21 / seafood)

#### • **Business context**

Import from the Republic of Belarus under mutual recognition of identification codes will be available. This means the marking system will receive, process, and recognize marking codes issued in the marking system of the Republic of Belarus and specified in the cross-border shipment documents received from the Operator of the Republic of Belarus.

The import process will be implemented as follows:

- 1. An exporter from the EEU generates the "Shipment from EEU with IC recognition" document (EAS\_CROSSBORDER). The document must contain the transferred marking codes and their attributes.
- 2. An importer in the Russian Federation generates the "Acceptance of shipment from EEU" document (LP\_ACCEPT\_GOODS) based on the received shipment document, selecting one of the following options:
  - full acceptance;
  - partial acceptance;
  - refusal of acceptance.
- 3. In the "Acceptance of shipment from EEU" document (LP\_ACCEPT\_GOODS), the importer in the Russian Federation can edit attributes for each marking code transferred in the shipment document.

After the "Acceptance of shipment from EEU" document (LP\_ACCEPT\_GOODS) is successfully processed, the importer becomes the owner of the marking codes, and each code's status changes to In circulation.

#### • **True API changes**

✅ The document to be available: "Acceptance of shipment from EEU" document (LP\_ACCEPT\_GOODS) submitted in response to the "Shipment from EEU with IC recognition" document (EAS\_CROSSBORDER)

#### • **Details**

Parameter requirements for the "Acceptance of shipment from EEU" document ( LP\_ACCEPT\_GOODS) submitted in response to the "Shipment from EEU with IC recognition" document (EAS\_CROSSBORDER) will match those for imports from the Republic of Armenia.

For the description of the documents, see [True API.](https://честныйзнак.рф/upload/docs/True_API_en.pdf)

## <span id="page-19-0"></span>**▪️ Export of canned foods and seafood under mutual recognition of identification codes to the Republic of Belarus**

### • **Goods groups affected**

- Canned foods (32 / conserve)
- Seafood (21 / seafood)

#### • **Business context**

A new process of export to the Republic of Belarus under mutual recognition of marking codes will become available:

- 1. An exporter transfers marked goods within the Russian Federation using the "Shipment to EEU" document (EAS\_CROSSBORDER\_EXPORT) of \* .json format.
- 2. An importer from an EEU country submits an acceptance document with details of the accepted goods specified in the shipment document. After successful processing of the document, the accepted marking codes will become available for circulation in the territory of the EEU country, and the exporter from the Russian Federation will be able to view the "Acceptance in EEU" document (EAS\_CROSSBORDER\_EXPORT\_ACCEPTANCE).

#### • **True API changes**

✅ The "Shipment to EEU" document (EAS\_CROSSBORDER\_EXPORT) will become available

#### • **Details**

Parameter requirements for the "Shipment to EEU" document (EAS\_CROSSBORDER\_EXPORT) will match those for exports to the Republic of Armenia.

For the description of the documents, see [True API.](https://честныйзнак.рф/upload/docs/True_API_en.pdf)

## <span id="page-20-0"></span>**▪️ Mandatory business place indication when withdrawing veterinary medicines from circulation**

#### • **Goods groups affected**

◦ Veterinary medicines (26 / vetpharma)

#### • **Business context**

Specifying business place details will become mandatory when withdrawing goods from circulation for certain reasons. This will improve tracking accuracy and ensure compliance with marking requirements.

#### • **True API changes**

ὐ The document to be modified: "Withdrawal from circulation" (LK\_RECEIPT)

#### • **Details**

In the "Withdrawal from circulation" document (LK\_RECEIPT), specifying business place details using the kpp (KPP of the goods circulation participant) and fias\_id (FIAS identifier) parameters will become mandatory if a goods circulation participant has [controlled facilities according to FGIS](https://docs.crpt.ru/gismt/Инструкция_по_редактированию_профиля/?search=редактирование%20данных%20участника#_редактирование_данных_участника) [VetIS data](https://docs.crpt.ru/gismt/Инструкция_по_редактированию_профиля/?search=редактирование%20данных%20участника#_редактирование_данных_участника) and the action parameter (Disposal reason) has one of the following values:

- DONATION (Compensation-free transfer)
- OTHER (Other)
- VETERINARY\_USE (Use for veterinary purposes)
- PRODUCTION\_USE (Use for production purposes)
- OWN\_USE (Use for internal purposes)
- EXPIRATION (Expiration)
- CONFISCATION (Confiscation)
- DESTRUCTION (Destruction)

- UTILIZATION (Disposal)
- LOSS (Loss)

For the description of the document, see [True API.](https://честныйзнак.рф/upload/docs/True_API_en.pdf)

## <span id="page-21-0"></span>**▪️ Universal messages in EDM Lite**

#### • **Goods groups affected**

◦ All goods groups

#### • **Business context**

EDM Lite will support creating universal messages for incoming and outgoing documents. This will allow you to promptly inform counterparties of document statuses (signed, clarified, canceled) and will streamline workflow coordination in electronic document management.

#### • **True API changes**

✅ Methods to be available:

- "[Method to create a universal message for an incoming / outgoing document](#page-21-1)"
- "[Method to create a universal message for a buyer's title / cancellation proposal"](#page-25-0)

#### <span id="page-21-1"></span>**▪️ Method to create a universal message for an incoming / outgoing document**

The method is intended to create a universal message (UM) via API using XML for an incoming / outgoing document.

#### **Type of privacy:** private

#### **URL:**

| Path                          | Description                                     |
|-------------------------------|-------------------------------------------------|
| /api/v3/true-api/edo/incoming | For buyer's title / cancellation of an incoming |
| documents/xml/unimsg/{doc_id} | document                                        |
| /api/v3/true-api/edo/outgoing | For buyer's title / cancellation of an outgoing |
| documents/xml/unimsg/{doc_id} | document                                        |

**Method:** POST

#### **Request string example:**

```
curl --location --request POST '<URL of environment>/api/v3/true-api/edo/incoming-
documents/xml/unimsg/{doc_id}'
--header 'authorization: ET <Token>'
--header 'content-type: multipart/form-data'
--form 'content=@/C:/Users//Desktop/ON.xml'
```

#### **Request string parameters:**

| Parameter | Type   | Mandat<br>ory | Description   | Comment |
|-----------|--------|---------------|---------------|---------|
| doc_id    | string | +             | The ID of the |         |
|           |        |               | document for  |         |
|           |        |               | which a UM is |         |
|           |        |               | being created |         |

#### **Request body parameters:**

| Parameter | Type | Mandat<br>ory | Description                       | Comment |
|-----------|------|---------------|-----------------------------------|---------|
| content   | file | +             | The XML file<br>containing the UM |         |

#### **Response parameters:**

| Parameter | Type   | Mandat<br>ory | Description                 | Comment |
|-----------|--------|---------------|-----------------------------|---------|
| id        | string | +             | The ID of the<br>created UM |         |

#### 1. **Example of a successful response (codes 200, 201):**

```
{
  "id":"cd0ce000-0cd0-0c0c-0f0a-00000e0c000f"
}
```

### 2. **Possible errors:**

| Error code | Error message                                                                                       | Error description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
|------------|-----------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 400        | Exchanging of a UM with the {value<br>from<br>Документ.СвСобытДок.СтатусУС}<br>status is restricted | Invalid UM code                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| 400        | Creation of a UM for the document is<br>not possible                                                | Creation of the УС-ИзвПол (Universal<br>message-Notice of receipt) (11519991)<br>and УС-комментариев (Universal<br>message-comments) (11540021) is<br>available for incoming documents in<br>any status. For outgoing documents,<br>only УС-комментариев (Universal<br>message-comments) (11540021) can be<br>created.<br>Creation of the УС-УвТоч (Universal<br>message-Notice of clarification) for<br>seller's title (11529991) is available<br>only for incoming documents with the<br>following statuses:<br>2<br>—<br>Signature is not required;<br>3<br>—<br>Signature is required;<br>4<br>—<br>Signed;<br>7<br>—<br>Clarified;<br>12<br>—<br>Signature is not required<br>(document is viewed);<br>13<br>—<br>Signature is required (document is<br>viewed);<br>18<br>—<br>Canceled;<br>19<br>—<br>Cancellation was refused;<br>61<br>—<br>Signed (sent to TT GIS);<br>62<br>—<br>Signed (not accepted by TT GIS);<br>63<br>—<br>Signed (Is being sent to TT GIS);<br>64<br>—<br>Canceled: Sent to TT GIS;<br>65<br>—<br>Canceled: Not accepted by TT<br>GIS;<br>66<br>—<br>Canceled: Is being sent to TT<br>GIS. |

| Error code | Error message                                                                                                     | Error description                                                                                                                                                                                                                             |  |  |
|------------|-------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--|--|
|            |                                                                                                                   | Creation of the УС-Отказ на НФ документ (Universal message-rejection on unformalized document) (11539991) is available only for incoming documents (unformalized documents with need_approval: true) with the following statuses:             |  |  |
|            |                                                                                                                   | 3 - Signature is required; 4 - Signed; 13 - Signature is required (document is viewed); 18 - Canceled; 19 - Cancellation was refused.                                                                                                         |  |  |
|            |                                                                                                                   | Creation of УС-Отказов для формализованных документов (Universal message-Rejections on formalized documents) and УС-УвТоч для НФ документов (Universal message-Notice of clarification for unformalized documents) is not currently supported |  |  |
| 400        | Invalid request body                                                                                              | Request body is missing                                                                                                                                                                                                                       |  |  |
| 400        | A value of the ИдОтпрУС element (UM sender ID) does not match the sender details in ИдФайл (File ID)              | Sender and receiver details in the UM                                                                                                                                                                                                         |  |  |
| 400        | A value of the ИдПолучУС element (UM receiver ID) does not match the receiver details in ИдФайл (File ID)         | file do not match the details in the UM filename                                                                                                                                                                                              |  |  |
| 400        | UM sender details {Identifier of the UM sender} do not match details of the receiver of the document {filename}   | Sender and receiver details in the UM file do not match sender and receiver details from the parent document                                                                                                                                  |  |  |
| 400        | UM receiver details {Identifier of the UM receiver} do not match details of the sender of the document {filename} |                                                                                                                                                                                                                                               |  |  |

| Error code | Error message                                                                                        | Error description                                                                                                       |
|------------|------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------|
| 400        | File<br>{Документ.СведУС.ИмяПолФайл}<br>referenced by UM could not be found                          | Document file name in the database<br>differs from the document referenced by<br>the UM<br>(Документ.СведУС.ИмяПолФайл) |
| 400        | Uploaded file name: {filename} does<br>not match ИдФайл (File ID) attribute<br>value: {ИдФайл value} | UM filename differs from UM ИдФайл<br>(File ID)                                                                         |
| 401        | Full authentication is required to access<br>this resource. Check your token                         | Invalid / expired token                                                                                                 |
| 403        | The role specified in the token does not<br>have permission for this document                        | Token contains a role that does not<br>authorize this request                                                           |
| 404        | Document could not be found in<br>database                                                           | A UTD for which the universal message<br>is being created could not be found                                            |
| 500        | Error processing request                                                                             | Internal server error                                                                                                   |

#### <span id="page-25-0"></span>**▪️ Method to create a universal message for a buyer's title / cancellation proposal**

The method is intended to create a universal message via API using XML for a buyer's title and cancellation proposal of an incoming / outgoing document.

#### **Type of privacy:** private

#### **URL:**

| Path                                                                           | Description                                                 |
|--------------------------------------------------------------------------------|-------------------------------------------------------------|
| /api/v3/true-api/edo/incoming<br>documents/xml/unimsg/{doc_id}/event/{event_id | For buyer's title / cancellation of an incoming<br>document |
| }                                                                              |                                                             |
| /api/v3/true-api/edo/outgoing                                                  | For buyer's title / cancellation of an outgoing             |
| documents/xml/unimsg/{doc_id}/event/{event_id                                  | document                                                    |
| }                                                                              |                                                             |

#### **Method:** POST

#### **Request string example:**

curl --location --request POST '<URL of environment>/api/v3/true-api/edo/incomingdocuments/xml/unimsg/{doc\_id}/event/{event\_id} '

```
--header 'authorization: ET <Token>'
--header 'content-type: multipart/form-data'
--form 'content=@/C:/Users//Desktop/ON.xml'
```

#### **Request string parameters:**

| Parameter | Type   | Mandat<br>ory | Description                                                     | Comment |
|-----------|--------|---------------|-----------------------------------------------------------------|---------|
| doc_id    | string | +             | The ID of the<br>document for<br>which a UM is<br>being created |         |
| event_id  | string | +             | The ID of the<br>receipt for which a<br>UM is being<br>created  |         |

#### **Request body parameters:**

| Parameter | Type | Mandat<br>ory | Description                       | Comment |
|-----------|------|---------------|-----------------------------------|---------|
| content   | file | +             | The XML file<br>containing the UM |         |

#### **Response parameters:**

| Parameter | Type   | Mandat<br>ory | Description                 | Comment |
|-----------|--------|---------------|-----------------------------|---------|
| id        | string | +             | The ID of the<br>created UM |         |

#### 1. **Response example (code 201 if successful):**

```
{
  "id":"cd0ce000-0cd0-0c0c-0f0a-00000e0c000f"
}
```

#### 2. **Possible errors:**

| Error code<br>Error message |                                                                                                                            | Error description                                                                                                     |  |
|-----------------------------|----------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------|--|
| 400                         | Exchanging of a UM with the<br>{value from<br>Документ.СвСобытДок.Статус<br>УС} status is restricted                       | Invalid UM code                                                                                                       |  |
| 400                         | Receiving a UM for buyer's title<br>with the {value from<br>Документ.СвСобытДок.Статус<br>УС} status is restricted         | Invalid (unsupported) UM code                                                                                         |  |
| 400                         | Receiving a UM for cancellation<br>proposal with the {value from<br>Документ.СвСобытДок.Статус<br>УС} status is restricted | specified for the receipt                                                                                             |  |
| 400                         | UM cannot be created for the<br>document                                                                                   |                                                                                                                       |  |
| 400                         | Invalid input parameters                                                                                                   |                                                                                                                       |  |
| 400                         | Invalid request body                                                                                                       | Request body is missing                                                                                               |  |
| 400                         | A value of the ИдОтпрУС<br>element (UM sender ID) does<br>not match sender details in<br>ИдФайл (File ID)                  | Sender and receiver details in the<br>UM file do not match the sender                                                 |  |
| 400                         | A value of the ИдПолучУС<br>element (UM receiver ID) does<br>not match receiver details in<br>ИдФайл (File ID)             | and receiver details in the UM<br>filename                                                                            |  |
| 400                         | UM sender details {Identifier of<br>the UM sender} do not match<br>details of the receiver of the<br>document {filename}   | Sender and receiver details in the<br>UM file do not match sender and<br>receiver details from the parent<br>document |  |
| 400                         | UM receiver details {Identifier<br>of the UM receiver} do not<br>match details of the sender of the<br>document {filename} |                                                                                                                       |  |

| Error code | Error message                                                                                                                 | Error description                                                                                                           |  |  |
|------------|-------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------|--|--|
| 400        | UM sender details {Identifier of<br>the UM sender} do not match the<br>cancellation proposal {filename}<br>receiver details   | For cancellation refusals, sender<br>and receiver details in the UM                                                         |  |  |
| 400        | UM receiver details {Identifier<br>of the UM receiver} do not<br>match the cancellation proposal<br>{filename} sender details | file are compared with the<br>cancellation proposal receiver<br>and sender details                                          |  |  |
| 400        | UM references receipt {value<br>from<br>Документ.СведУС.ИмяПолФа<br>йл} and cannot be uploaded to<br>receipt {filename}       | Receipt file number in the<br>database differs from the receipt<br>referenced by the UM<br>(Документ.СведУС.ИмяПолФа<br>йл) |  |  |
| 400        | Uploaded file name: {filename}<br>does not match ИдФайл (File<br>ID) attribute value: {ИдФайл<br>value}                       | UM filename differs from UM<br>ИдФайл (File ID)                                                                             |  |  |
| 401        | Full authentication is required to<br>access this resource. Check your<br>token                                               | Invalid / expired token                                                                                                     |  |  |
| 403        | The role specified in the token<br>does not have permission for this<br>document                                              | Token contains a role that does<br>not authorize this request                                                               |  |  |
| 404        | Document could not be found in<br>the database / Receipt could not<br>be found in the database                                |                                                                                                                             |  |  |
| 500        | Error processing request                                                                                                      | Internal server error                                                                                                       |  |  |

## <span id="page-28-0"></span>**▪️ Obtaining an authorization token in a new format for True API methods**

#### • **Goods groups affected**

◦ All goods groups

#### • **Business context**

The process for obtaining a single authentication token in UUID format will be updated. This improvement will allow you to switch from the JWT format to the shorter UUID format.

JWT token support will be maintained until the end of 2026.

#### • **True API changes**

- ὐ The following methods will be updated:
  - "Requesting authorization in single authentication" (/auth/key)
  - "Receiving the session key in single authentication" (/auth/simpleSignIn)

#### • **Details**

Planned changes for the "Requesting authorization in single authentication" method (/auth/key):

- to obtain an authorization token in UUID format, you must request a unique pair using the "Requesting authorization in single authentication" method (/auth/key): a UUID (an identifier of the current authentication) and generated random data signed with the EQES of the organization user making a request;
- support for the current UUID token logic (without a preliminary request for a unique pair using the "Requesting authorization in single authentication" method (/auth/key)) will be maintained until September 2026.

Planned changes for the "Receiving the session key in single authentication" method (/auth/simpleSignIn):

- for users switching from JWT to UUID authorization tokens:
  - requirements for filling in the request body parameter for obtaining the token will be changed:

| Parameter | Type   | Manda<br>tory | Description      | Comment                             |
|-----------|--------|---------------|------------------|-------------------------------------|
| inn       | string | -             | INN of the       | INN of the organization must be     |
|           |        |               | organization     | specified if the individual         |
|           |        |               | requiring        | (organization user) has a valid     |
|           |        |               | authorization.   | machine-readable PoA for TT GIS     |
|           |        |               | Length of value: | access. Otherwise, the parameter is |
|           |        |               | 10 or 12 digits  | optional.                           |

▪ a request body parameter will be added:

| Parameter   | Type    | Manda<br>tory | Description                                                        | Comment                                                                                                                                                                                                                         |
|-------------|---------|---------------|--------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| unitedToken | boolean | -             | Indicator that a<br>single token is<br>requested in<br>UUID format | The parameter is mandatory when<br>requesting a UUID token.<br>Possible values:<br>true — to request a token in UUID<br>format (single authentication<br>token);<br>false — to request a token in JWT<br>format (default value) |

▪ response parameters will be added:

| Parameter  | Type   | Manda<br>tory | Description                               | Comment                                                                               |
|------------|--------|---------------|-------------------------------------------|---------------------------------------------------------------------------------------|
| uuidToken  | string | -             | Authentication<br>token in UUID<br>format | The parameter is returned in a<br>successful response if a UUID<br>token is requested |
| expireDate | string | -             | Token expiration<br>date and time         | The parameter is returned only if a<br>UUID token is requested.<br>Format: yyyy-MM    |
|            |        |               |                                           | ddTHH:mm:ss.SSSZ                                                                      |
|            |        |               |                                           | Example: 2026-10-<br>10T00:00:00.123Z                                                 |

- for users already using UUID authorization tokens:
  - requirements for filling in the request body parameters for obtaining the token will be changed:

| Parameter | Type   | Description                                                                                                                                            | Current                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | New                                                                                                                                                                                                     |
|-----------|--------|--------------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|           |        |                                                                                                                                                        | (UUID)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | (UUID)                                                                                                                                                                                                  |
| uuid      | string | Unique<br>identifier of<br>signed<br>random data                                                                                                       | Optional                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | Mandatory                                                                                                                                                                                               |
| data      | string | Random<br>data signed<br>with the<br>registered<br>goods<br>circulation<br>participant's<br>EQES in<br>base64<br>(attached<br>electronic<br>signature) | If "unitedToken" = "true",<br>one<br>of<br>the<br>following<br>values is provided:<br>•<br>signed<br>INN<br>of<br>the<br>goods<br>circulation<br>participant for which<br>authorization<br>is<br>requested,<br>if<br>it<br>is<br>necessary to obtain a<br>token for a user acting<br>on<br>behalf<br>of<br>the<br>organization under the<br>machine-readable<br>PoA;<br>•<br>signed data (Random<br>data string) obtained<br>from the "Requesting<br>Authorization<br>in<br>Single<br>Authentication"<br>(/auth/key)<br>method<br>or signed INN of the<br>goods<br>circulation<br>participant,<br>if<br>it<br>is<br>necessary to obtain a<br>user token based on<br>EQES<br>for<br>a<br>legal<br>entity<br>/<br>individual<br>entrepreneur<br>or<br>depersonalized EQES | Signed data (Random<br>data string) obtained from<br>the "Requesting<br>Authorization in Single<br>Authentication" method<br>(/auth/key) in base64<br>(attached electronic<br>signature) is transmitted |

| Parameter | Type   | Description                                                                                                                    | Current<br>(UUID)                                                    | New<br>(UUID)                                                                                                                                                                                                      |
|-----------|--------|--------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| inn       | string | INN of the<br>organization<br>for which<br>user<br>authorizatio<br>n is<br>required.<br>Length of<br>value: 10 or<br>12 digits | If "unitedToken" = "true",<br>the parameter must not be<br>filled in | INN of the organization<br>must be specified if an<br>individual (organization<br>user) has a valid machine<br>readable power of<br>attorney to access the TT<br>GIS. In other cases, the<br>parameter is optional |

◦ a response parameter will be added:

| Parameter  | Type   | Mandat<br>ory | Description                       | Comment                                                                                                                              |
|------------|--------|---------------|-----------------------------------|--------------------------------------------------------------------------------------------------------------------------------------|
| expireDate | string | -             | Token expiration<br>date and time | It is returned only when requesting a<br>UUID token.<br>Format: yyyy-MM<br>ddTHH:mm:ss.SSSZ<br>Example: 2026-10-<br>10T00:00:00.123Z |

- the following additional specifics regarding token usage will be implemented:
  - UUID token lifetime will be 10 hours from issuance. The lifetime cannot exceed the expiration date of the EQES and/or machine-readable PoA. If a machine-readable PoA under which the user is granted access to the system is revoked, the token will be automatically invalidated;
  - a new token will be issued for each request (existing token lifetime will not be extended).

For the description of the methods, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf).

<span id="page-33-0"></span>**▪️ Import of biologically active food additives, veterinary medicines, perfume and eau de toilette, cosmetics, household chemicals, and personal hygiene products from the Republic of Armenia under mutual recognition of identification codes**

#### • **Goods groups affected**

- Biologically active food additives (17 / bio)
- Veterinary medicines (26 / vetpharma)
- Perfume and eau de toilette (4 / perfumery)
- Cosmetics, household chemicals, and personal hygiene products (35 / chemistry)

#### • **Business context**

Import from the Republic of Armenia will be available within the framework of mutual recognition of identification codes. This means that the marking system will ensure receipt, processing, and recognition of marking codes issued in the marking system of the Republic of Armenia and specified in the cross-border shipment document received from the Operator of the Republic of Armenia.

Import will be implemented as follows:

- 1. An exporter from the EEU generates the "Shipment from EEU with IC recognition" document (EAS\_CROSSBORDER) containing the transferred marking codes and their attributes.
- 2. An importer in the Russian Federation generates the "Acceptance of shipment from EEU" document (LP\_ACCEPT\_GOODS) based on the received shipment document, selecting one of the following options:
  - full acceptance;
  - partial acceptance;
  - refusal of acceptance.
- 3. In the "Acceptance of shipment from EEU" document (LP\_ACCEPT\_GOODS) the importer in the Russian Federation can edit attributes for each marking code transferred in the shipment document.

After successful processing of the "Acceptance of shipment from EEU" document (LP\_ACCEPT\_GOODS), the importer becomes the owner of the marking codes, and their status changes to In circulation.

#### • **True API changes**

✅ The document to be available: "Acceptance of shipment from EEU" document

(LP\_ACCEPT\_GOODS) submitted in response to the "Shipment from EEU with IC recognition" document (EAS\_CROSSBORDER)

#### • **Details**

Distinctive features that are planned to be implemented:

- the certificate\_document\_data parameter (Array containing data from documents confirming compliance) will be mandatory for the "Biologically active food additives" and "Veterinary medicines" goods groups;
- submission of the document will be available only in \* .json format.

For a description of the documents, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf).

## <span id="page-34-0"></span>**▪️ Export of biologically active food additives, veterinary medicines, perfume and eau de toilette, cosmetics, household chemicals, and personal hygiene products to the Republic of Armenia**

#### • **Goods groups affected**

- Biologically active food additives (17 / bio)
- Veterinary medicines (26 / vetpharma)
- Perfume and eau de toilette (4 / perfumery)
- Cosmetics, household chemicals, and personal hygiene products (35 / chemistry)

#### • **Business context**

A new process of export to the Republic of Armenia will be implemented under the mutual recognition of marking codes:

- 1. Marked goods are transferred on the territory of the Russian Federation by an exporter using the "Shipment to EEU" document (EAS\_CROSSBORDER\_EXPORT) in \* .json format.
- 2. An importer from an EEU country submits an acceptance document that contains information on the accepted goods specified in the shipment document. Once the document has been processed successfully, the accepted marking codes will become available for circulation on the territory of the EEU country, and the exporter from the Russian Federation will be able to view the "Acceptance in EEU" document (EAS\_CROSSBORDER\_EXPORT\_ACCEPTANCE).

#### • **True API changes**

✅ The "Shipment to EEU" document (EAS\_CROSSBORDER\_EXPORT) will become available

#### • **Details**

Distinctive features that are planned to be implemented:

◦ the following parameters will be added to the products objects array (Array containing a list of transferred ICs):

| Parameter     | Type    | Mandat<br>ory | Description                        | Comment                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
|---------------|---------|---------------|------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| *batch_number | string  | -             | Batch number                       | The parameter is mandatory for the<br>"Veterinary medicines" goods group.<br>For other goods groups, the<br>parameter must not be filled in. The<br>value must not exceed 30 characters<br>including spaces (digits, Cyrillic and<br>Latin letters, special characters (/ . ,<br>-))                                                                                                                                                                                                                        |
| *alco_content | decimal | -             | Actual content of<br>ethyl alcohol | The parameter is mandatory for the<br>"Cosmetics,<br>household<br>chemicals,<br>and<br>personal<br>hygiene<br>products"<br>goods group, for other goods groups<br>the parameter must not be filled in.<br>Format:<br>•<br>up to 3 digits in the integer part<br>and up to 1 digit in the decimal<br>part<br>(decimal<br>separator<br>is<br>a<br>period, decimal part is optional);<br>•<br>value ranges from 0 (0.0) to 100<br>(100.0) inclusive.<br>If it is alcohol-free, "0" value shall<br>be specified |

<sup>◦</sup> the production\_date parameter (Production date) is mandatory for the "Biologically active food additives", "Veterinary medicines", "Cosmetics, household chemicals, and personal hygiene products" goods groups, and is optional for the "Perfume and Eau de Toilette" goods group;

- the expiration\_date parameter (Expiration date) is mandatory for the "Biologically active food additives", "Veterinary medicines", "Cosmetics, household chemicals, and personal hygiene products" goods groups, and must not be filled in for the "Perfume and Eau de Toilette" goods group;
- the certificate\_document\_data parameter (Array containing data from documents confirming compliance) is mandatory for the "Veterinary medicines" goods group;
- submission of the document will be available only in \* .json format.

For a description of the documents, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf).

## <span id="page-36-0"></span>**▪️ Simplified process of import of engine oils from the Kyrgyz Republic**

#### • **Goods groups affected**

◦ Engine oils (43 / autofluids)

#### • **Business context**

A simplified process of *engine oils* import from the Kyrgyz Republic to the Russian Federation is planned for implementation.

The following scheme will be implemented to import goods under the simplified process:

- 1. An exporter from the Kyrgyz Republic describes a goods item in the National catalog and orders Russian marking codes via the Marking system of the Kyrgyz Republic.
- 2. The exporter then applies the marking codes onto goods and submits information on the crossborder movement of the goods to the Marking system of the Kyrgyz Republic. The importer can view this information in the Marking system of the Russian Federation in the "Shipment from EEU" document (LP\_SHIP\_GOODS\_CROSSBORDER).
- 3. The importer submits the "Acceptance" document (LP\_ACCEPT\_GOODS) containing details of the goods acceptance as specified in the shipment document to the Marking system of the Russian Federation. Once the acceptance document is successfully processed, the accepted marking codes will be introduced into circulation.

#### • **True API changes**

✅ The document to be available: "Acceptance of shipment from EEU" (LP\_ACCEPT\_GOODS) in response to the "Shipment from EEU" document (LP\_SHIP\_GOODS\_CROSSBORDER)

#### • **Details**

Requirements for filling in parameters in the "Acceptance of shipment from EEU" document (LP\_ACCEPT\_GOODS) will be similar to those used when importing the goods from the Republic of Belarus.

For a description of the documents, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf).

## <span id="page-37-0"></span>**▪️ Obtaining virtual warehouse information for canned foods**

#### • **Goods groups affected**

◦ Canned foods (32 / conserve)

#### • **Business context**

For *canned foods*, you will be able to obtain the virtual warehouse information via the True API export generation method. This will allow goods circulation participants to monitor stock levels and track movement of goods within the warehouse.

#### • **True API changes**

✅ Method to be available: "Obtaining information about virtual warehouse, including batch accounting" (dispenser/tasks)

## <span id="page-37-1"></span>**▪️ Mandatory indication of the date range when requesting a list of documents**

#### • **Goods groups affected**

◦ All goods groups

#### • **Business context**

When requesting a list of documents, it will be necessary to specify a date range for which data are requested, or to indicate additional filtering parameters.

This change is intended to improve the stability and performance of the method when processing large volumes of documents.

#### • **True API changes**

ὐ The method to be modified: "Method of receiving a list of documents loaded in TT GIS" (/api/v4/true-api/doc/list)

#### • **Details**

The logic of using the dateFrom (Document date, from) and dateTo (Document date, to) request string parameters will be changed as follows:

- 1. Both parameters must be provided together. Providing only one of them is not allowed.
- 2. The parameters will be mandatory if the request string:
  - includes none of the following parameters: documentStatus (Document status), documentType (Document type), number (Document identifier);
  - and does not include either the pair of documentFormat (Document format) and senderInn (INN of the document sender) or the pair of documentFormat and receiverInn (INN of the document receiver).

Additionally, the allowed date range between dateFrom and dateTo will be limited.

If these conditions are not met, the method will return an error.

For a description of the method, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf).

## <span id="page-38-0"></span>**▪️ Filtering dairy products by shipment number in the virtual warehouse**

#### • **Goods groups affected**

◦ Dairy products (8 / milk)

#### • **Business context**

Shipment information will be added to the virtual warehouse. This will allow you to track *dairy products* by specific shipment number.

#### • **True API changes**

ὐ The methods to be modified:

- "Method of obtaining the current balance in a warehouse" (/warehouse/balance)
- "Method of getting a list of operations in a warehouse" (/warehouse/operations)

#### • **Details**

The following will be added to the "Method of obtaining the current balance in a warehouse" method (/warehouse/balance):

◦ optional request parameters:

| Parameter     | Type                | Mandat<br>ory | Description                     | Comment                                                                                                                                                                                                    |
|---------------|---------------------|---------------|---------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| partNumbers   | array of<br>strings | -             | Array of<br>shipment<br>numbers | The parameter is used for the "Dairy<br>products" goods group.<br>Maximum: 50 shipment numbers                                                                                                             |
| customGroupBy | array of<br>strings | -             | Array of grouping<br>indicators | The parameter is used for the "Dairy<br>products" goods group.<br>It is mandatory if the partNumbers<br>(Array of shipment numbers) is filled<br>in. Possible grouping value:<br>partNumber (Shipment No.) |

◦ optional response parameter in the balances array:

| Parameter   | Type   | Mandat<br>ory | Description  | Comment                                                                                                                                                                                              |
|-------------|--------|---------------|--------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| *partNumber | string | -             | Shipment No. | The parameter is returned if the<br>request body contained the<br>partNumbers (Array of shipment<br>numbers) and / or customGroupBy<br>(Array of grouping indicators) =<br>partNumber (Shipment No.) |

The following will be added to the "Method of getting a list of operations in a warehouse" (/warehouse/operations):

#### ◦ optional request parameters:

| Parameter   | Type                | Mandat<br>ory | Description                     | Comment                                                        |
|-------------|---------------------|---------------|---------------------------------|----------------------------------------------------------------|
| partNumbers | array of<br>strings | -             | Array of<br>shipment<br>numbers | The parameter is used for the "Dairy<br>products" goods group. |
|             |                     |               |                                 | Maximum: 50 shipment numbers                                   |

| Parameter     | Type                | Mandat<br>ory | Description                     | Comment                                                                                                                                  |
|---------------|---------------------|---------------|---------------------------------|------------------------------------------------------------------------------------------------------------------------------------------|
| customGroupBy | array of<br>strings | -             | Array of grouping<br>indicators | The parameter is used for the "Dairy<br>products" goods group.                                                                           |
|               |                     |               |                                 | It is mandatory if the partNumbers<br>(Array of shipment numbers) is filled<br>in. Possible grouping value:<br>partNumber (Shipment No.) |

◦ optional response parameter in the operations array:

| Parameter   | Type   | Mandat<br>ory | Description  | Comment                                                                                                                                                                                   |
|-------------|--------|---------------|--------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| *partNumber | string | -             | Shipment No. | It is returned if the request body<br>contained the partNumbers (Array<br>of shipment numbers) and / or<br>customGroupBy (Array of grouping<br>indicators) = partNumber (Shipment<br>No.) |

For a description of the methods, see [True API.](https://честныйзнак.рф/upload/docs/True_API_en.pdf)

## <span id="page-40-0"></span>**▪️ Manage consumer feedback**

#### • **Goods groups affected**

◦ All goods groups

#### • **Business context**

In the near future you will be able to directly handle consumers feedback submitted via Chestny ZNAK mobile app, including:

- receive a list of complaints related to your goods;
- see details: description, photos, information about goods and points of sale;
- send official responses with explanation and attached photos.

How this will work:

1. A consumer identifies a potential issue — such as damaged, expired goods, or error in the marking — and submits feedback via the Chestny ZNAK mobile app.

2. The system identifies which circulation participant is responsible under the agreement and

redirects the complaint to that participant.

3. You (as the responsible participant) review the complaint, investigate the issue, and submit a

response.

4. The Operator moderates the response:

▪ if the response is constructive and substantive - the consumer sees it in the app;

▪ if the response does not contain explanations or deviates from the topic - the case is

escalated to public authorities (e.g., Rospotrebnadzor).

This makes you the first to review and respond to consumer complaints — delivering significant

advantages:

◦ Resolve quality, marking, and sale conditions issues faster;

◦ Reduce risks - identify supply chain problems at an early stage;

◦ Close complaints faster - without the participation of public authorities;

◦ Protect your brand reputation - react promptly and show your position;

◦ Build consumer trust - demonstrate transparency and customer care;

◦ Access analytics to track the number of complaints, affected goods, and their regional

distribution.

Access to this feature will be provided upon signing the agreement for handling consumer

feedback. You may submit a connection request to Technical Support after the feature is launched.

• **True API changes**

✅ New methods to be available:

◦ "[Method to get a list of consumer complaints"](#page-41-0)

◦ "[Method to view consumer complaint](#page-46-0)"

◦ "[Method to send a response to consumer complaint](#page-50-0)"

<span id="page-41-0"></span>**▪️ Method to get a list of consumer complaints**

The method is intended to get a list of consumers' complaints submitted via the Chestny ZNAK mobile

app.

**Type of privacy:** private

42

**Method:** POST

#### **Request body example:**

```
{
  "filter":{
  "statuses":[
  "PENDING",
  "NOT_RESPONDED"
  ],
  "regionFiasId":[
  "string"
  ],
  "categories":[
  "tobacco"
  ],
  "kinds":[
  "NO_MARKING"
  ],
  "checkDateFrom":1750511770,
  "checkDateTo":1750511780
  },
  "pageable":{
  "sort":"-checkDate",
  "page":1,
  "pageSize":20
  }
}
```

#### **Request body parameters:**

| Parameter      | Type                | Mandat<br>ory | Description                                                 | Comment                                                                                    |
|----------------|---------------------|---------------|-------------------------------------------------------------|--------------------------------------------------------------------------------------------|
| filter         | object              | -             | Filtering<br>parameters for<br>searching<br>complaints      |                                                                                            |
| *statuses      | array of<br>strings | -             | A list of status<br>codes of<br>complaints<br>consideration | See "Catalog "Statuses of complaint<br>consideration by goods circulation<br>participant"" |
| *regionFiasIds | array of<br>strings | -             | A list of identifiers<br>of regions<br>according to FIAS    | Current values can be found on the<br>website of Federal Information Address<br>System     |

| Parameter      | Type                | Mandat<br>ory | Description                                                | Comment                                                                                                                                                                                                                                                                                                         |
|----------------|---------------------|---------------|------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| *categories    | array of<br>strings | -             | A list of codes of<br>the goods groups                     | See "Catalog "List of supported goods<br>groups""                                                                                                                                                                                                                                                               |
| *kinds         | array of<br>strings | -             | A list of codes of<br>complaint types                      | See "Catalog "Types of consumer<br>complaints""                                                                                                                                                                                                                                                                 |
| *checkDateFrom | long                | -             | Start date for<br>filtering the<br>complaints              | Format: timestamp in seconds                                                                                                                                                                                                                                                                                    |
| *checkDateTo   | long                | -             | End date for<br>filtering the<br>complaints                | Format: timestamp in seconds                                                                                                                                                                                                                                                                                    |
| pageable       | object              | -             | Parameters to work<br>with the final list of<br>complaints |                                                                                                                                                                                                                                                                                                                 |
| *sort          | string              | -             | The parameter and<br>direction of sorting                  | Possible values:<br>number<br>—<br>by violation number<br>(ascending);<br>-number<br>—<br>by violation number<br>(descending);<br>checkDate<br>—<br>by check date from the<br>lowest to the highest date;<br>-checkDate<br>—<br>by check date from the<br>highest to the lowest date.<br>Default value: -number |
| *page          | integer             | -             | Number of the<br>requested page                            | Default value: 1                                                                                                                                                                                                                                                                                                |
| *pageSize      | integer             | -             | Number of<br>complaints per<br>page                        | Default value<br>—<br>20. Maximum allowed<br>value is 1,000                                                                                                                                                                                                                                                     |

#### 1. **Response example (code 200 if successful):**

```
{
  "content":[
```

```
  {
  "number":154366,
  "checkDate":1750598163,
  "status":"PENDING",
  "category":"tobacco",
  "inn":"123123123",
  "kind":"NO_MARKING",
  "regionFiasId":"1ac11b11-1111-1111-b1bf-a111ea1aecd1",
  "regionName":"Ивановская область",
  "address":"630520, Новосибирская обл, Новосибирский р-н, село Верх-Тула,
ул Советская, д 77777"
  },
  {
  "number":154532,
  "checkDate":1750511775,
  "status":"RESPONSE_APPROVED",
  "category":"milk",
  "inn":"123123123",
  "kind":"EXPIRED_GOODS",
  "regionFiasId":"1ac11b11-1111-1111-b1bf-a111ea1aecd1",
  "regionName":"Ивановская область",
  "address":"630520, Новосибирская обл, Новосибирский р-н, село Верх-Тула,
ул Советская, д 77777"
  }
  ],
  "totalElements":2,
  "innAgreement":true
}
```

#### 2. **Response examples in case of errors:**

◦ Code 400 if \* .json structure in request body is invalid:

```
{
  "error_message":"JSON parse error: <информация-парсера>"
}
```

◦ Code 401 if invalid or expired token is specified:

```
{
  "error_message":"Токен не действителен. Необходимо получить новый токен
аутентификации"
}
```

◦ Code 403 if there is no access to resource:

```
{
```

```
  "error_message":"Отсутствует доступ к ресурсу"
}
```

◦ Code 4ХХ: linked service error

```
{
  "error_message":"<сообщение об ошибке outer из параметра errors>"
}
```

◦ Code 5ХХ: internal server error

```
{
  "error_message":"<сообщение об ошибке true-api>"
}
```

#### **Response parameters:**

| Parameter  | Type                | Mandat<br>ory | Description                                                 | Comment                                                                                    |
|------------|---------------------|---------------|-------------------------------------------------------------|--------------------------------------------------------------------------------------------|
| content    | array of<br>objects | -             | Array of<br>complaints objects<br>provided to the<br>seller |                                                                                            |
| *number    | long                | -             | Complaint No.                                               |                                                                                            |
| *checkDate | long                | -             | Check date and<br>time                                      | Format: timestamp in seconds                                                               |
| *status    | string              | -             | A status code of<br>complaint<br>consideration by<br>seller | See "Catalog "Statuses of complaint<br>consideration by goods circulation<br>participant"" |
| *category  | string              | -             | Goods group code                                            | See "Catalog "List of supported goods<br>groups""                                          |
| *inn       | string              | -             | Seller's INN (from<br>complaint)                            |                                                                                            |
| *kind      | string              | -             | Complaint type<br>code                                      | See "Catalog "Types of consumer<br>complaints""                                            |

| Parameter     | Type    | Mandat<br>ory | Description                                                    | Comment                                                                                |
|---------------|---------|---------------|----------------------------------------------------------------|----------------------------------------------------------------------------------------|
| *regionFiasId | string  | -             | Region ID (FIAS)                                               | Current values can be found on the<br>website of Federal Information Address<br>System |
| *regionName   | string  | -             | Region name                                                    |                                                                                        |
| *address      | string  | -             | Address of a point<br>of sale                                  |                                                                                        |
| totalElements | long    | -             | Total number of<br>comlaints meeting<br>search conditions      |                                                                                        |
| innAgreement  | boolean | -             | Availability of<br>consent to handle<br>consumer<br>complaints | Possible values:<br>true (Consent granted);<br>false (No consent)                      |

#### <span id="page-46-0"></span>**▪️ Method to view consumer complaint**

The method is intended to get full information about a complaint for its further consideration by the seller.

**Type of privacy:** private

**Method:** GET

#### **Request string parameters:**

| Parameter | Type | Mandat<br>ory | Description                 | Comment |
|-----------|------|---------------|-----------------------------|---------|
| number    | long | +             | No. of desired<br>complaint |         |

#### 1. **Response example (code 200 if successful):**

```
{
  "number":154532,
  "checkDate":1750511775,
  "category":"milk",
  "inn":"123123123",
  "kind":"NO_MARKING",
  "comment":"Это не масло, а маргарин",
```

```
  "salePoint":{
  "name":"АО \"Предприятие\"",
  "inn":"123123123",
  "regionFiasId":"1ac11b11-1111-1111-b1bf-a111ea1aecd1",
  "houseFiasId":"1ac11b11-1111-1111-b1bf-a111ea1aecd1",
  "regionName":"Ивановская область",
  "address":"630520, Новосибирская обл, Новосибирский р-н, село Верх-Тула, ул
Советская, д 77777",
  "marketplace":null
  },
  "product":{
  "category":"milk",
  "code":"0104644444444444215K\"PaE\\u001d9315x5",
  "productLink":null
  },
  "photos":[
  "https://mobile.api.crpt.ru/photos/motp-photo.jpg",
  "https://mobile.api.crpt.ru/external/asset/motp.png"
  ],
  "partnerReview":{
  "status":"PENDING",
  "date":1750511790,
  "resolution":"В рамках выявленного нарушения мы убрали просроченный товар",
  "photos":[
  "string"
  ]
  }
}
```

#### 2. **Response examples in case of errors:**

◦ Code 400 if the number parameter has not been transferred:

```
{
  "error_message":"Метод с указанным URL не найден"
}
```

◦ Code 400 if a request string structure is invalid:

```
{
  "error_message":"Failed to read HTTP message"
}
```

◦ Code 401 if a user is not authenticated:

```
{
  "error_message":"Для доступа к этому ресурсу требуется полная аутентификация.
```

```
Проверьте токен" }
```

## ° Code 5XX: internal server error

```
{
    "error_message":"<описание ошибки ...>"
}
```

### **Response parameters:**

| Parameter            | Type   | Mandat<br>ory | Description                           | Comment                                        |
|----------------------|--------|---------------|---------------------------------------|------------------------------------------------|
| number               | long   | -             | Complaint No.                         |                                                |
| checkDate            | long   | -             | Check date and time                   | Format: timestamp in seconds                   |
| category             | string | -             | Goods group code                      | See "Catalog "List of supported goods groups"" |
| inn                  | string | -             | Seller's INN (from complaint)         |                                                |
| kind                 | string | -             | Complaint type code                   | See "Catalog "Types of consumer complaints""   |
| comment              | string | -             | Consumer's comment text               |                                                |
| moderatorCommen<br>t | string | -             | Moderator's comment                   |                                                |
| product              | object | -             | Details of the product from complaint |                                                |
| *category            | string | -             | Goods group code                      | See "Catalog "List of supported goods groups"" |
| *code                | string | -             | Identification code of the goods item |                                                |
| *name                | string | -             | Product name                          |                                                |

| Parameter     | Type                | Mandat<br>ory | Description                                                     | Comment                                                                                    |
|---------------|---------------------|---------------|-----------------------------------------------------------------|--------------------------------------------------------------------------------------------|
| *productLink  | string              | -             | A link to the goods<br>item in the online<br>store              |                                                                                            |
| salePoint     | object              | -             | Details of a point<br>of sale                                   |                                                                                            |
| *name         | string              | -             | Name of a point of<br>sale                                      |                                                                                            |
| *inn          | string              | -             | INN of a point of<br>sale                                       |                                                                                            |
| *regionFiasId | string              | -             | Region ID<br>according to FIAS                                  | Current values can be found on the<br>website of Federal Information Address<br>System     |
| *regionName   | string              | -             | Region name                                                     |                                                                                            |
| *houseFiasId  | string              | -             | Identifier of the<br>address (house)<br>according to FIAS       |                                                                                            |
| *address      | string              | -             | Address of a point<br>of sale                                   |                                                                                            |
| *marketplace  | string              | -             | Marketplace code                                                |                                                                                            |
| photos        | array of<br>strings | -             | Array of photos of<br>the complaint                             |                                                                                            |
| partnerReview | object              | -             | Object of the<br>results of<br>consideration by<br>seller       |                                                                                            |
| *status       | string              | -             | A status code of<br>the complaint<br>consideration              | See "Catalog "Statuses of complaint<br>consideration by goods circulation<br>participant"" |
| *date         | long                | -             | Date and time of<br>the complaint<br>consideration by<br>seller | Format: timestamp in seconds                                                               |

| Parameter   | Type                | Mandat<br>ory | Description                                                                        | Comment |
|-------------|---------------------|---------------|------------------------------------------------------------------------------------|---------|
| *resolution | string              | -             | The seller's<br>comment that has<br>been sent when<br>considering the<br>complaint |         |
| *photos     | array of<br>strings | -             | Photos from the<br>seller provided as<br>links to data<br>storage                  |         |

#### <span id="page-50-0"></span>**▪️ Method to send a response to consumer complaint**

The method is intended to send a goods circulation participant's response to a consumer complaint.

**Type of privacy:** private

**Method:** PATCH

#### **Request string parameters:**

| Parameter | Type | Mandat<br>ory | Description   | Comment |
|-----------|------|---------------|---------------|---------|
| number    | long | +             | Complaint No. |         |

#### **Request body example:**

```
{
  "status":"ACKNOWLEDGED",
  "resolution":"В рамках выявленного нарушения мы убрали просроченный товар"
}
```

#### **Request body parameters:**

| Parameter  | Type   | Mandat<br>ory | Description                                                     | Comment                                                                                               |
|------------|--------|---------------|-----------------------------------------------------------------|-------------------------------------------------------------------------------------------------------|
| status     | string | +             | A status code of<br>complaint<br>consideration by<br>the seller | Possible values:<br>ACKNOWLEDGED<br>—<br>complaint<br>confirmed;<br>DENIED<br>—<br>complaint rejected |
| resolution | string | +             | Seller's comment<br>text                                        | Maximum length of the value: 1,000<br>characters (including spaces)                                   |

- 1. **In case of success, response status 204 with empty body will be returned.**
- 2. **Response examples in case of errors:**
  - Code 400 If mandatory request parameter is missing:

```
{
  "error_message":"Отсутствует обязательный параметр <имя параметра>"
}
```

◦ Code 400 if the number parameter has not been transferred:

```
{
  "error_message":"Метод с указанным URL не найден"
}
```

◦ Code 400 If the request body is missing:

```
{
  "error_message":"Тело запроса не может быть пустым"
}
```

◦ Code 400 if \* .json structure in request body is invalid:

```
{
  "error_message":"JSON parse error: <информация-парсера>"
}
```

◦ Code 401 if an invalid or expired token is specified:

```
{
```

```
  "error_message":"Токен не действителен. Необходимо получить новый токен
аутентификации"
}
```

◦ Code 403 if no access to resource has been found:

```
{
  "error_message":"Отсутствует доступ к ресурсу"
}
```

◦ Code 4ХХ: linked service error

```
{
  "error_message":"<сообщение об ошибке outer из параметра errors>"
}
```

◦ Code 5ХХ: internal server error

```
{
  "error_message":"<сообщение об ошибке true-api>"
}
```

## <span id="page-52-0"></span>**▪️ Shipment of dairy products to the Republic of Belarus**

• **Goods groups affected**

◦ Dairy products (8 / milk)

• **Business context**

For the *dairy products* you will be able to submit information on shipment of goods to the Republic of Belarus by using the "Shipment to EEU" document (EAS\_CROSSBORDER\_EXPORT) in \* .json format.

• **True API changes**

✅ Document to be opened: "Shipment to EEU" (EAS\_CROSSBORDER\_EXPORT)

#### • **Details**

Distinctive features that are planned to be implemented:

◦ the following parameters will be added into the products objects array (Array containing a list

#### of transferred ICs):

| Parameter       | Type    | Mandat<br>ory | Description                                     | Comment                                                                                                                                                                                                                                                                                                                                  |
|-----------------|---------|---------------|-------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| *part_number    | string  | -             | Shipment No.                                    | It may be filled in for the "Dairy<br>products" goods group, for other<br>goods groups it must be unfilled.<br>Format: from 21 to 32 characters,<br>inclusive with spaces. Allowed<br>characters: digits, Latin letters,<br>special characters (/ . , -)                                                                                 |
| *product_weight | integer | -             | Variable weight<br>of the product (in<br>grams) | It is mandatory for the "Dairy<br>products" goods group, if<br>isVarQuantity (Goods with<br>variable quantity) = true in a goods<br>card in the NCMG. For other goods<br>groups the parameter is optional. The<br>parameter shall be missing for the<br>goods without variable weight.<br>Range of values: from 1 to 999999<br>inclusive |

<sup>◦</sup> conditions of mandatory of the production\_date (Production date) and expiration\_date (Expiration date) parameters will be similar to those applicable to the "Packaged water" goods group.

For a description of the document, see [True API.](https://честныйзнак.рф/upload/docs/True_API_en.pdf)

#### • **Date of the announcement**

Announced in v.430.0 dated June 19, 2025

## <span id="page-53-0"></span>**▪️ Closing the "Withdrawal from circulation" document for export of light industry to the Republic of Belarus**

#### • **Goods groups affected**

◦ Light industry (1 / lp)

#### • **Business context**

The ability to indicate the Republic of Belarus as a destination country will be closed when submitting information about export via the "Withdrawal from circulation" document (LK\_RECEIPT, LK\_RECEIPT\_CSV, LK\_RECEIPT\_XML)

#### • **True API changes**

❌ The "Withdrawal from circulation" document (LK\_RECEIPT, LK\_RECEIPT\_CSV, LK\_RECEIPT\_XML) will be closed for export to the Republic of Belarus

#### • **Details**

The ability to indicate information about the Republic of Belarus will be unavailable in the destination\_country\_code (Code of the destination country) and importer\_id (INN (or analog) of the importer) parameters in the "Withdrawal from circulation" document type (LK\_RECEIPT, LK\_RECEIPT\_CSV, LK\_RECEIPT\_XML).

To export goods to the Republic of Belarus, use the "Shipment to EEU" document (EAS\_CROSSBORDER\_EXPORT, EAS\_CROSSBORDER\_EXPORT\_CSV).

For a description of the documents, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf).

## <span id="page-54-0"></span>**▪️ Improvement of the method of receiving information on goods item by goods GTIN (v4)**

#### • **Goods groups affected**

◦ All goods groups

#### • **True API changes**

ὐ Modified method: "Method of receiving information on goods item by goods GTIN" of version v4.

#### • **Details**

The "Method of receiving information on goods item by goods GTIN" of version v4 is being improved:

◦ The logic of issue of information for the GTIN attributes will be changed. To date, information is returned only on the codes in the goods cards of which values of the goodTurnFlag (Indicator of goods readiness to the circulation) and goodMarkFlag (Indicator of readiness to the marking) attributes equal true. In a new version information about goods will be returned regardless of these attributes.

◦ A new response parameter - attributes (Object of strings with data about attributes) will be added. This parameter will contain information that describes goods from the NCMG:

| Parameter                     | Type                | Mandat<br>ory | Description                                                                           | Comment                                                                                                                                                                                                   |
|-------------------------------|---------------------|---------------|---------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| *attributes                   | array of<br>string  | +             | Object of strings<br>with data about<br>attributes                                    |                                                                                                                                                                                                           |
| **<наименовани<br>е атрибута> | array of<br>string  | +             | Goods<br>description<br>attribute                                                     | The resulting * .json can contain a<br>set of fields specific for the goods of<br>the certain goods group (see<br>"Catalog - Additional parameters in<br>the response, depending on the goods<br>groups") |
| ***data                       | array of<br>objects | +             | Array of objects<br>with the attributes<br>data that returned<br>from the NC /<br>GS1 |                                                                                                                                                                                                           |
| ****value                     | string              | +             | Value of the<br>attribute from the<br>NCMG                                            |                                                                                                                                                                                                           |
| ****type                      | string              | -             | Type / unit of<br>measurement of<br>the attribute from<br>the NCMG                    |                                                                                                                                                                                                           |

For a description of the method, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf).

# <span id="page-56-0"></span>**Appendix 1. Catalogs**

## <span id="page-56-1"></span>**Catalog "List of supported goods groups"**

| Code in DB | Name        | Description                                                       |
|------------|-------------|-------------------------------------------------------------------|
| 1          | lp          | Light industry                                                    |
| 2          | shoes       | Footwear                                                          |
| 3          | tobacco     | Tobacco products                                                  |
| 4          | perfumery   | Perfume and Eau de Toilette                                       |
| 5          | tires       | New pneumatic rubber tires and tire casings                       |
| 6          | electronics | Photo cameras (except cine cameras), flash lights and flash bulbs |
| 8          | milk        | Dairy products                                                    |
| 9          | bicycle     | Bicycles and bicycle frames                                       |
| 10         | wheelchairs | Medical products                                                  |
| 11         | alcohol     | Alcohol                                                           |
| 12         | otp         | Alternative tobacco products                                      |
| 13         | water       | Packaged water                                                    |
| 14         | furs        | Products made of real fur                                         |
| 15         | beer        | Beer, beer-based and low-alcohol beverages                        |
| 16         | ncp         | Nicotine products                                                 |
| 17         | bio         | Specialized food products and biologically active food additives  |
| 19         | antiseptic  | Antiseptic/antibacterial skin cleansers and hand sanitizers       |
| 20         | petfood     | Pet foods                                                         |
| 21         | seafood     | Seafood                                                           |
| 22         | nabeer      | Non-alcoholic beer                                                |
| 23         | softdrinks  | Juice products and non-alcoholic beverages                        |
| 25         | meat        | Meat products                                                     |
| 26         | vetpharma   | Veterinary medicines                                              |
| 27         | toys        | Games and toys for children                                       |
| 28         | radio       | Radio-electronic products                                         |

| Code in DB | Name         | Description                                                     |
|------------|--------------|-----------------------------------------------------------------|
| 31         | titan        | Titanium metal products                                         |
| 32         | conserve     | Canned foods                                                    |
| 33         | vegetableoil | Vegetable oils                                                  |
| 34         | opticfiber   | Optical fiber and fiber optic products                          |
| 35         | chemistry    | Perfumes and toilet preparations and household chemicals        |
| 36         | books        | Printed goods                                                   |
| 37         | grocery      | Groceries                                                       |
| 38         | pharmaraw    | Pharmaceutical raw materials, medicines                         |
| 39         | construction | Building materials                                              |
| 40         | fire         | Pyrotechnics and fire-fighting equipment                        |
| 41         | heater       | Heaters                                                         |
| 42         | cableraw     | Cabling and wiring products                                     |
| 43         | autofluids   | Engine oils                                                     |
| 44         | polymer      | Polymer pipes                                                   |
| 45         | sweets       | Confectionery products                                          |
| 48         | carparts     | Auto parts and components for vehicles                          |
| 49         | furslp       | Real fur                                                        |
| 50         | nicotindev   | Radio-electronic products. Electronic nicotine delivery systems |
| 51         | gadgets      | Radio-electronic products. Laptops and smartphones              |
| 52         | frozen       | Semi-finished and frozen food products                          |
| 53         | fertilizers  | Fertilizers in consumer packaging                               |
| 54         | homeware     | Home and kitchen goods                                          |

## <span id="page-57-0"></span>**Catalog "Statuses of complaint consideration by goods circulation participant"**

| Code    | Name                   | Description                                             |
|---------|------------------------|---------------------------------------------------------|
| PENDING | Awaiting consideration | The complaint is awaiting the seller's<br>consideration |

| Code          | Name                    | Description                                     |
|---------------|-------------------------|-------------------------------------------------|
| ACKNOWLEDGED  | Complaint accepted      | The seller acknowledges the violation           |
| DENIED        | Complaint rejected      | The seller does not agree with the<br>violation |
| NOT_RESPONDED | No response is provided | The seller has not provided a response          |

## <span id="page-58-0"></span>**Catalog "Types of consumer complaints"**

| Code                   | Name                                           |
|------------------------|------------------------------------------------|
| NO_MARKING             | Goods are not marked                           |
| MARKING_NOT_READ       | Marking cannot be read                         |
| EXPIRED_GOODS          | Expired goods item                             |
| INVALID_DOCS           | Invalid permitting document                    |
| OTHER                  | Other                                          |
| DIFFER_INFO            | App info does not correspond to the packaging  |
| COMPLAINT_PHARMACY     | Complaint against pharmacy                     |
| WITHDRAW               | Repeated sales                                 |
| NONE                   | No violation type                              |
| COMPLAINT_MARKETPLACE  | Complaint against marketplace                  |
| VAD_VIOLATIONS         | Violations in veterinary accompanying document |
| IGNORE_PERMISSION_MODE | Ignoring the permit regime                     |

## <span id="page-58-1"></span>**Catalog "Groups of permitting documents statuses"**

| Status group          | PD statuses                                                                                                                                                                                                                                                                                                                                                                                                                             | Description                                                                                                                      |
|-----------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------|
| 1                     | Active<br>(Certificate<br>of<br>conformity<br>/<br>Declaration of conformity);<br>Signed<br>and<br>active<br>(State<br>registration<br>certificate);<br>Renewed<br>(Certificate<br>of<br>conformity<br>/<br>Declaration of conformity);<br>Extended<br>(Certificate<br>of<br>conformity<br>/<br>Declaration of conformity);<br>Awaiting verification by registry operator<br>(Certificate of conformity / Declaration of<br>conformity) | Green status<br>—<br>goods that have such a<br>permitting document are available for<br>introduction, circulation and withdrawal |
| 4                     | Unknown status                                                                                                                                                                                                                                                                                                                                                                                                                          |                                                                                                                                  |
| 26, 27, 28, 29,<br>30 | Status assigned during emergency operation<br>mode                                                                                                                                                                                                                                                                                                                                                                                      |                                                                                                                                  |

| Status group                             | PD statuses                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | Description                                                                                                                                                                                                                                    |
|------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 2                                        | Suspended<br>(Certificate<br>of<br>conformity<br>/<br>Declaration of conformity / State registration<br>certificate);<br>Improvement notice issued (Certificate of<br>conformity / Declaration of conformity);<br>Termination notice sent;<br>Terminated<br>(Certificate<br>of<br>conformity<br>/<br>Declaration of conformity) not by decision of<br>regulatory authorities;<br>Deleted<br>(reissued)<br>(State<br>registration<br>certificate);<br>Archived<br>(if<br>not<br>previously<br>invalid)<br>(Certificate of conformity / Declaration of<br>conformity);<br>Annulled / Withdrawn (if withdrawn by an<br>applicant) (State registration certificate) | Yellow status<br>—<br>goods that have such a<br>permitting document are available for<br>circulation and withdrawal, but they are<br>unavailable for primary introduction<br>(during the production and import into<br>the Russian Federation) |
| 5                                        | Status assigned to the Russian state<br>registration certificate not found in the<br>Rospotrebnadzor registry                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |                                                                                                                                                                                                                                                |
| 6                                        | Status assigned to EEU state registration<br>certificate not found in the Eurasian<br>Economic Commission registry                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |                                                                                                                                                                                                                                                |
| 7                                        | Status assigned to Russian certificate of<br>conformity / declaration of conformity not<br>found in the RusAccreditation registry                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |                                                                                                                                                                                                                                                |
| 9                                        | Status assigned to EEU certificate of<br>conformity / declaration of conformity not<br>found in the Eurasian Economic Commission<br>registry                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |                                                                                                                                                                                                                                                |
| 11                                       | Status assigned when a permitting document<br>expires                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |                                                                                                                                                                                                                                                |
| 12, 13, 14, 15,<br>21, 22, 23, 24,<br>25 | Status assigned during emergency operation<br>mode                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |                                                                                                                                                                                                                                                |

| Status group          | PD statuses                                                                                                                                                                                                                                                                                                                                                                                   | Description                                                                                                                      |
|-----------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------|
| 3                     | Annulled<br>/<br>Revoked<br>(if<br>annulled<br>by<br>regulatory<br>authorities)<br>(State<br>registration<br>certificate);<br>Invalid<br>(Certificate<br>of<br>conformity<br>/<br>Declaration of conformity);<br>Terminated<br>(Certificate<br>of<br>conformity<br>/<br>Declaration of conformity) by decision of<br>regulatory authorities;<br>Archived (if previously invalid) (Certificate | Red status<br>—<br>goods that have such a<br>permitting document are unavailable for<br>introduction, circulation and withdrawal |
|                       | of conformity / Declaration of conformity)                                                                                                                                                                                                                                                                                                                                                    |                                                                                                                                  |
| 16, 17, 18, 19,<br>20 | Status assigned during emergency operation<br>mode                                                                                                                                                                                                                                                                                                                                            |                                                                                                                                  |

# <span id="page-62-0"></span>**Modifications introduced in the previous versions of the document**

| NOTE         | This section contains records for the last three months from the last publication of the<br>document. For earlier records, see archive |
|--------------|----------------------------------------------------------------------------------------------------------------------------------------|
|              | v.551.0 dated May 28, 2026                                                                                                             |
| Planned:     |                                                                                                                                        |
|              | ▪️ Working with industrial marking goods codes through the sub-account configuration mechanism.                                        |
| Implemented: |                                                                                                                                        |
|              | ▪️ Withdrawal of radio-electronic products from circulation for prepacking operations.                                                 |
|              | For the details of implementation, see True API of v.671.0 dated May 28, 2026.                                                         |
|              | v.550.0 dated May 26, 2026                                                                                                             |
| Planned:     |                                                                                                                                        |
|              | ▪️ Mandatory indication of previous code when remarking biologically active food additives.                                            |
|              | v.549.0 dated May 25, 2026                                                                                                             |
| Implemented: |                                                                                                                                        |
|              | ▪️ Introducing goods received from a principal into circulation for real fur.                                                          |

v.548.0 dated May 22, 2026

### Planned:

▪️ Specifying AVD when introducing [confectionery products](#page-5-1) and [semi-finished and frozen food](#page-6-0) [products](#page-6-0) into circulation.

For the details of implementation, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf) of v.669.0 dated May 25, 2026.

| Implemented:                                                                                                                                                                                                                                      |
|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| ▪️ Remarking of laptops and smartphones, as well as radio-electronic products.                                                                                                                                                                    |
| For the details of implementation, see True API of v.668.0 dated May 22, 2026.                                                                                                                                                                    |
| v.547.0 dated May 21, 2026                                                                                                                                                                                                                        |
| Implemented:                                                                                                                                                                                                                                      |
| ▪️ Specifying medical device model for marketing authorization.<br>▪️ Simplified import process for radio-electronic products from the Republic of Belarus.                                                                                       |
| For the details of implementation, see True API of v.667.0 dated May 21, 2026.                                                                                                                                                                    |
| v.546.0 dated May 19, 2026                                                                                                                                                                                                                        |
| Planned:                                                                                                                                                                                                                                          |
| ▪️ Withdrawal of radio-electronic products from circulation for prepacking operations.                                                                                                                                                            |
| v.545.0 dated May 18, 2026                                                                                                                                                                                                                        |
| Planned:                                                                                                                                                                                                                                          |
| ▪️ Mandatory business place indication for withdrawal from circulation for certain goods groups.                                                                                                                                                  |
| v.544.0 dated May 15, 2026                                                                                                                                                                                                                        |
| Planned:                                                                                                                                                                                                                                          |
| ▪️ Adding and removing business places for engine oils.                                                                                                                                                                                           |
| Implemented:                                                                                                                                                                                                                                      |
| ▪️ Withdrawal of groceries from circulation (volume and grade accounting) and cancellation of such<br>withdrawal.<br>▪️ Withdrawal of groceries from circulation for prepacking operations.<br>▪️ Generation of multi-product shipping packaging. |

| For the details of implementation, see True API of v.664.0 dated May 15, 2026.                                                                                                                                                 |  |  |  |  |  |
|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--|--|--|--|--|
| v.543.0 dated May 14, 2026                                                                                                                                                                                                     |  |  |  |  |  |
| Planned:                                                                                                                                                                                                                       |  |  |  |  |  |
| ▪️ Remarking of laptops and smartphones, as well as radio-electronic products.                                                                                                                                                 |  |  |  |  |  |
| Partially implemented:                                                                                                                                                                                                         |  |  |  |  |  |
| ▪️ Simplified import process for laptops and smartphones from the Republic of Belarus.<br>▪️ Import and export of laptops and smartphones from/to the Republic of Belarus under mutual<br>recognition of identification codes. |  |  |  |  |  |
| For the details of implementation, see True API of v.663.0 dated May 14, 2026.                                                                                                                                                 |  |  |  |  |  |
| Next up: Simplified import process for laptops and smartphones from the Republic of Armenia.                                                                                                                                   |  |  |  |  |  |
| v.542.0 dated May 12, 2026                                                                                                                                                                                                     |  |  |  |  |  |
| Planned:                                                                                                                                                                                                                       |  |  |  |  |  |
| ▪️ Withdrawal of groceries from circulation (volume and grade accounting) and cancellation of such<br>withdrawal.                                                                                                              |  |  |  |  |  |
| ▪️ Withdrawal of groceries from circulation for prepacking operations.                                                                                                                                                         |  |  |  |  |  |
| ▪️ Simplified import process for radio-electronic products from the Republic of Belarus.                                                                                                                                       |  |  |  |  |  |
| v.541.0 dated May 6, 2026                                                                                                                                                                                                      |  |  |  |  |  |
| Implemented:                                                                                                                                                                                                                   |  |  |  |  |  |
| ▪️ Adding and removing business places for cosmetics, household chemicals, and personal hygiene<br>products, as well as real fur.                                                                                              |  |  |  |  |  |
| ▪️ Generating exports for laptops and smartphones.                                                                                                                                                                             |  |  |  |  |  |
| For the details of implementation, see True API of v.661.0 dated May 6, 2026.                                                                                                                                                  |  |  |  |  |  |
| v.540.0 dated May 5, 2026                                                                                                                                                                                                      |  |  |  |  |  |

| Implemented:                                                                                                             |
|--------------------------------------------------------------------------------------------------------------------------|
| ▪️ Disabling correction of remaining stock (volume and grade accounting) for pet foods.                                  |
| ▪️ Additional withdrawal reasons for laptops and smartphones.                                                            |
| For the details of implementation, see True API of v.660.0 dated May 5, 2026.                                            |
| v.539.0 dated May 4, 2026                                                                                                |
| Implemented:                                                                                                             |
| ▪️ Marking of remaining stock for laptops and smartphones, as well as radio-electronic products.                         |
| For the details of implementation, see True API of v.659.0 dated May 4, 2026.                                            |
| v.538.0 dated April 30, 2026                                                                                             |
| Planned:                                                                                                                 |
| ▪️ Generation of multi-product shipping packaging.                                                                       |
| Implemented:                                                                                                             |
| ▪️ Import via FCS and ACC support for laptops and smartphones, radio-electronic products, and<br>confectionery products. |
| ▪️ Correcting code details for confectionery products.                                                                   |
| ▪️ Additional withdrawal reason for toys.                                                                                |
| For the details of implementation, see True API of v.658.0 dated April 30, 2026.                                         |
| v.537.0 dated April 28, 2026                                                                                             |
| Planned:                                                                                                                 |
| ▪️ Generating exports for laptops and smartphones.                                                                       |
| v.536.0 dated April 27, 2026                                                                                             |

Implemented:

| ▪️ Adding and removing business places for some goods groups.                                                                                                                                                                                           |
|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| For the details of implementation, see True API of v.656.0 dated April 27, 2026.                                                                                                                                                                        |
| v.535.0 dated April 23, 2026                                                                                                                                                                                                                            |
| Planned:                                                                                                                                                                                                                                                |
| ▪️ EEU integration for laptops and smartphones.                                                                                                                                                                                                         |
| v.534.0 dated April 22, 2026                                                                                                                                                                                                                            |
| Planned:                                                                                                                                                                                                                                                |
| ▪️ Introducing goods received from a principal into circulation for real fur.<br>▪️ Import via FCS and ACC support for laptops and smartphones, radio-electronic products, and<br>confectionery products.<br>▪️ Migrating EDM Lite methods to True API. |
| v.533.0 dated April 21, 2026                                                                                                                                                                                                                            |
| Planned:                                                                                                                                                                                                                                                |
| ▪️ Exporting codes by permitting document status groups and validity periods.<br>▪️ Exporting codes missing required permitting documents.                                                                                                              |
| v.532.0 dated April 20, 2026                                                                                                                                                                                                                            |
| Planned:                                                                                                                                                                                                                                                |
| ▪️ Marking of remaining stock for laptops and smartphones, as well as radio-electronic products.<br>▪️ Specifying medical device model for marketing authorization.                                                                                     |
| Implemented:                                                                                                                                                                                                                                            |
| ▪️ Mandatory business place indication when withdrawing clothes and linens from circulation.                                                                                                                                                            |
| For the details of implementation, see True API of v.651.0 dated April 20, 2026.                                                                                                                                                                        |
|                                                                                                                                                                                                                                                         |

### v.531.0 dated April 17, 2026

![](api-v552.0-05.06.2026-at-13-03-25_images/_page_67_Figure_1.jpeg)

| ▪️ Additional withdrawal reasons for laptops and smartphones. |  |  |  |  |
|---------------------------------------------------------------|--|--|--|--|
|                                                               |  |  |  |  |

▪️ Disabling correction of remaining stock (volume and grade accounting) for pet foods.

### Change in plans:

▪️ Adding and removing business places for cosmetics, household chemicals, personal hygiene products, and real fur: the methods will also be available for a number of other goods groups.

### Implemented:

▪️ Discontinuation of real fur product remarking using ICM.

For the details of implementation, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf) of v.650.0 dated April 17, 2026.

### v.530.0 dated April 16, 2026

### Planned:

- ▪️ Correcting code details for confectionery products.
- ▪️ [Import of canned foods and seafood under mutual recognition of identification codes from the](#page-18-0) [Republic of Belarus](#page-18-0).
- ▪️ [Export of canned foods and seafood under mutual recognition of identification codes to the](#page-19-0) [Republic of Belarus](#page-19-0).
- ▪️ Additional withdrawal reason for toys.

### v.529.0 dated April 14, 2026

### Planned:

▪️ Adding and removing business places for cosmetics, household chemicals, personal hygiene products, and real fur.

### v.528.0 dated April 13, 2026

### Planned:

| ▪️ Mandatory business place indication when withdrawing veterinary medicines, clothes and linens<br>from circulation.                                               |
|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| v.527.0 dated April 10, 2026                                                                                                                                        |
| Planned:                                                                                                                                                            |
| ▪️ Universal messages in EDM Lite.                                                                                                                                  |
| Implemented:                                                                                                                                                        |
| ▪️ Lot number verification when forming a group package for dairy products.<br>▪️ Checking a shipment number when updating the code details for the dairy products. |
| For the details of implementation, see True API of v.647.0 dated April 10, 2026.                                                                                    |
| v.526.0 dated April 9, 2026                                                                                                                                         |
| Planned:                                                                                                                                                            |
| ▪️ Discontinuation of real fur product remarking using ICM.                                                                                                         |
| Implemented:                                                                                                                                                        |
| ▪️ Verification of permitting document details when remaining stock of toys is introduced into<br>circulation.                                                      |
| For the details of implementation, see True API of v.646.0 dated April 9, 2026.                                                                                     |
| v.525.0 dated April 3, 2026                                                                                                                                         |
| Planned:                                                                                                                                                            |
| ▪️ Obtaining an authorization token in a new format for True API methods.                                                                                           |
| Implemented:                                                                                                                                                        |
| ▪️ Detailed report on goods cards for building materials.                                                                                                           |
| For the details of implementation, see True API of v.644.0 dated April 3, 2026.                                                                                     |
|                                                                                                                                                                     |

### v.524.0 dated April 2, 2026

| Implemented: |  |
|--------------|--|
|              |  |

▪️ Marking of remaining stock of technical rehabilitation aids.

For the details of implementation, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf) of v.643.0 dated April 2, 2026.

### v.523.0 dated April 1, 2026

### Implemented:

- ▪️ Discontinuation of marking of remaining stock for engine oils.
- ▪️ Correction of information on codes of meat products, laptops and smartphones, and radio-electronic products.
- ▪️ Lot number verification when withdrawing dairy products from circulation within grade accounting.

For the details of implementation, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf) of v.642.0 dated April 1, 2026.

### v.522.0 dated March 31, 2026

### Implemented:

- ▪️ Generation of sets for some goods groups.
- ▪️ Generation of group packages for laptops and smartphones, semi-finished and frozen food products.
- ▪️ Import and export of groceries from/to the Republic of Armenia under mutual recognition of marking codes.

For the details of implementation, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf) of v.641.0 dated March 31, 2026.

### v.521.0 dated March 26, 2026

### Planned:

▪️ Marking of remaining stock of technical rehabilitation aids.

### v.520.0 dated March 25, 2026

### Planned:

| ▪️ Import of biologically active food additives, veterinary medicines, perfume and eau de toilette,<br>cosmetics, household chemicals, and personal hygiene products under mutual recognition of<br>identification codes from the Republic of Armenia.<br>▪️ Export of biologically active food additives, veterinary medicines, perfume and eau de toilette, |
|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| cosmetics, household chemicals, and personal hygiene products to the Republic of Armenia.                                                                                                                                                                                                                                                                     |
| v.519.0 dated March 24, 2026                                                                                                                                                                                                                                                                                                                                  |
| Planned:                                                                                                                                                                                                                                                                                                                                                      |
| ▪️ Generation of sets for some goods groups.<br>▪️ Verification of permitting document details when remaining stock of toys is introduced into<br>circulation.<br>▪️ Lot number verification when withdrawing dairy products from circulation within grade accounting.<br>▪️ Lot number verification when forming a group package for dairy products.         |
| Change in plans:                                                                                                                                                                                                                                                                                                                                              |
| ▪️ Checking a shipment number when updating the code details for the dairy products: an IC must not<br>be nested into a GPIC or SIC.                                                                                                                                                                                                                          |
| v.518.0 dated March 23, 2026                                                                                                                                                                                                                                                                                                                                  |
| Implemented:                                                                                                                                                                                                                                                                                                                                                  |
| ▪️ Disabling set reaggregation for radio-electronic products (electronic nicotine delivery systems).                                                                                                                                                                                                                                                          |
| For the details of implementation, see True API of v.635.0 dated March 23, 2026.                                                                                                                                                                                                                                                                              |
| v.517.0 dated March 20, 2026                                                                                                                                                                                                                                                                                                                                  |
| Planned:                                                                                                                                                                                                                                                                                                                                                      |
| ▪️ Detailed report on goods cards for building materials.                                                                                                                                                                                                                                                                                                     |
| v.516.0 dated March 19, 2026                                                                                                                                                                                                                                                                                                                                  |
| Planned:                                                                                                                                                                                                                                                                                                                                                      |

| ▪️ Discontinuation of marking of remaining stock for engine oils.<br>▪️ Correction of information on codes of meat products, laptops and smartphones, and radio-electronic<br>products. |
|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| v.515.0 dated March 17, 2026                                                                                                                                                            |
| Implemented:                                                                                                                                                                            |
| ▪️ Obtaining information about the virtual warehouse for canned foods via the following methods:                                                                                        |
| •<br>"Method of obtaining the current balance in a warehouse" (/warehouse/balance);                                                                                                     |
| •<br>"Method of getting a list of operations in a warehouse" (/warehouse/operations).                                                                                                   |
| For the details of implementation, see True API of v.632.0 dated March 17, 2026.                                                                                                        |
| v.514.0 dated March 16, 2026                                                                                                                                                            |
| Planned:                                                                                                                                                                                |
| ▪️ Generation of group packages for laptops and smartphones, semi-finished and frozen food products.<br>▪️ Simplified process of import of engine oils from the Kyrgyz Republic.        |
| v.513.0 dated March 11, 2026                                                                                                                                                            |
| Planned:                                                                                                                                                                                |
| ▪️ Disabling set reaggregation for radio-electronic products (electronic nicotine delivery systems).                                                                                    |
| v.512.0 dated March 6, 2026                                                                                                                                                             |
| Planned:                                                                                                                                                                                |
| ▪️ Obtaining virtual warehouse information for canned foods.                                                                                                                            |
| v.511.0 dated March 4, 2026                                                                                                                                                             |
| Implemented:                                                                                                                                                                            |

| ▪️ Withdrawal of canned foods from circulation in terms of volume and grade and cancellation of such<br>withdrawal.<br>▪️ Obtaining minimum price via True API methods for alternative tobacco products.                                                         |
|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| For the details of implementation, see True API of v.626.0 dated March 4, 2026.                                                                                                                                                                                  |
| v.510.0 dated March 3, 2026                                                                                                                                                                                                                                      |
| Implemented:                                                                                                                                                                                                                                                     |
| ▪️ Disabling the "Obtaining information on identification codes and aggregates" export.                                                                                                                                                                          |
| For the details of implementation, see True API of v.625.0 dated March 3, 2026.                                                                                                                                                                                  |
| v.509.0 dated March 2, 2026                                                                                                                                                                                                                                      |
| Implemented:                                                                                                                                                                                                                                                     |
| ▪️ Changes in operations related to fur products.<br>▪️ Import of confectionery products via the FCS and use of the ACC.<br>▪️ Completion of remaining item marking for antiseptics.<br>▪️ Additional reasons for withdrawal from circulation for meat products. |
| For the details of implementation, see True API of v.624.0 dated March 2, 2026.                                                                                                                                                                                  |
| v.508.0 dated February 27, 2026                                                                                                                                                                                                                                  |
| Implemented:                                                                                                                                                                                                                                                     |
| ▪️ Withdrawal of cosmetics, household chemicals, and personal hygiene products from circulation in<br>terms of volume and grade and cancellation of such withdrawal.                                                                                             |
| For the details of implementation, see True API of v.623.0 dated February 27, 2026.                                                                                                                                                                              |
| v.507.0 dated February 27, 2026                                                                                                                                                                                                                                  |
| Planned:                                                                                                                                                                                                                                                         |
| ▪️ Obtaining minimum price via True API methods for alternative tobacco products.                                                                                                                                                                                |

| Implemented:                                                                                                                                                                                                                      |
|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| ▪️ Changes to the list of clothing and linen eligible for marking of remaining stock.<br>▪️ Simplified process of import of pet foods, cosmetics, household chemicals, and personal hygiene<br>products from the Kyrgyz Republic. |
| For the details of implementation, see True API of v.622.0 dated February 27, 2026.                                                                                                                                               |
| v.506.0 dated February 20, 2026                                                                                                                                                                                                   |
| Planned:                                                                                                                                                                                                                          |
| ▪️ Changes in operations related to fur products.<br>▪️ Changes to the list of clothing and linen eligible for marking of remaining stock.<br>▪️ Additional reasons for withdrawal from circulation for meat products.            |
| v.505.0 dated February 18, 2026                                                                                                                                                                                                   |
| Planned:                                                                                                                                                                                                                          |
| ▪️ Import of confectionery products via the FCS and use of the ACC.                                                                                                                                                               |
| v.504.0 dated February 17, 2026                                                                                                                                                                                                   |
| Planned:                                                                                                                                                                                                                          |
| ▪️ Completion of remaining item marking for antiseptics.<br>▪️ Withdrawal of canned foods from circulation in terms of volume and grade and cancellation of such<br>withdrawal.                                                   |
| Implemented:                                                                                                                                                                                                                      |
| ▪️ Adding and removing business places for some goods groups.                                                                                                                                                                     |
| For the details of implementation, see True API of v.619.0 dated February 17, 2026.                                                                                                                                               |
| v.503.0 dated February 13, 2026                                                                                                                                                                                                   |

Implemented:

| ▪️ Ability to create sets for radio-electronic products (electronic nicotine delivery systems).    |
|----------------------------------------------------------------------------------------------------|
| For the details of implementation, see True API of v.618.0 dated February 13, 2026.                |
|                                                                                                    |
| v.502.0 dated February 12, 2026                                                                    |
| Planned:                                                                                           |
| ▪️ Withdrawal of cosmetics, household chemicals, and personal hygiene products from circulation    |
| (volume and grade accounting).                                                                     |
| ▪️ Simplified process of import of pet foods, cosmetics, household chemicals, and personal hygiene |
| products from the Kyrgyz Republic.                                                                 |
| ▪️ Adding and removing business places for some goods groups.                                      |
|                                                                                                    |
|                                                                                                    |

### v.501.0 dated February 5, 2026

### Implemented:

▪️ Export of identification codes filtered by permitting documents.

For the details of implementation, see [True API](https://честныйзнак.рф/upload/docs/True_API_en.pdf) of v.615.0 dated February 5, 2026.