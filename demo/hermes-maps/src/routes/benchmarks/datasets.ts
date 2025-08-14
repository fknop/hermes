type Dataset = {
  category: string
  problems: string[]
}

export const CVRPTW_DATASETS: { category: string; datasets: Dataset[] }[] = [
  {
    category: 'Solomon',
    datasets: [
      {
        category: 'c1',
        problems: [
          'c101',
          'c102',
          'c103',
          'c104',
          'c105',
          'c106',
          'c107',
          'c108',
          'c109',
        ],
      },
      {
        category: 'c2',
        problems: [
          'c201',
          'c202',
          'c203',
          'c204',
          'c205',
          'c206',
          'c207',
          'c208',
        ],
      },
      {
        category: 'r1',
        problems: [
          'r101',
          'r102',
          'r103',
          'r104',
          'r105',
          'r106',
          'r107',
          'r108',
          'r109',
          'r110',
          'r111',
          'r112',
        ],
      },
      {
        category: 'r2',
        problems: [
          'r201',
          'r202',
          'r203',
          'r204',
          'r205',
          'r206',
          'r207',
          'r208',
          'r209',
          'r210',
          'r211',
        ],
      },
      {
        category: 'rc1',
        problems: [
          'rc101',
          'rc102',
          'rc103',
          'rc104',
          'rc105',
          'rc106',
          'rc107',
          'rc108',
        ],
      },
      {
        category: 'rc2',
        problems: [
          'rc201',
          'rc202',
          'rc203',
          'rc204',
          'rc205',
          'rc206',
          'rc207',
          'rc208',
        ],
      },
    ],
  },
]
